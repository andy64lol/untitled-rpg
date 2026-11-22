use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use macroquad::prelude::*;

use crate::assets::{draw_text_at, draw_wrapped_text, Assets};
use crate::Cli;
use crate::config;
use crate::config::*;
use crate::enemies::EnemyManager;
use crate::items::{draw_quality, Equipment, Inventory, Item, Stack};
use crate::map::{MapData, Tile};
pub use crate::map::log_line;
use crate::player::Player;
use crate::save::{self, SaveState};
use crate::script::{ScriptAction, ScriptContext, ScriptRunner};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Menu,
    Game,
    Won,
    Dead,
    SlotLoad,
    SlotSave,
    Settings,
}

const SETTING_START_MAP: usize = 0;
const SETTING_START_SCRIPT: usize = 1;
const SETTING_HEALTH: usize = 2;
const SETTING_FREECAM: usize = 3;
const SETTING_PLAYER_LIGHT: usize = 4;
const SETTING_MAP_LIGHT: usize = 5;
const SETTING_ROOMS: usize = 6;
const SETTING_FOG: usize = 7;
const SETTING_NO_ENEMIES: usize = 8;
const SETTING_DEBUG: usize = 9;
const SETTING_OUTPUT: usize = 10;
const SETTING_BACK: usize = 11;

#[derive(Clone, Debug)]
struct Dialogue {
    full: String,
    revealed: f32,
    open: bool,
    fast: bool,
}

impl Dialogue {
    fn new() -> Self {
        Self { full: String::new(), revealed: 0.0, open: false, fast: false }
    }

    fn show(&mut self, text: &str) {
        self.full = text.to_string();
        self.revealed = 0.0;
        self.fast = false;
        self.open = true;
    }

    fn visible(&self) -> String {
        let count = (self.revealed as usize).min(self.full.len());
        self.full.chars().take(count).collect()
    }

    fn complete(&self) -> bool {
        self.revealed >= self.full.chars().count() as f32
    }

    fn update(&mut self, dt: f32) {
        if !self.open || self.complete() {
            return;
        }
        let speed = if self.fast { 270.0 } else { 45.0 };
        self.revealed = (self.revealed + speed * dt)
            .min(self.full.chars().count() as f32);
    }

    fn finish(&mut self) {
        self.revealed = self.full.chars().count() as f32;
    }

    fn close(&mut self) {
        self.open = false;
        self.revealed = 0.0;
        self.fast = false;
    }
}

#[derive(Clone, Debug)]
struct Options {
    open: bool,
    choices: Vec<String>,
    index: usize,
    pending: String,
}

impl Options {
    fn new() -> Self {
        Self { open: false, choices: Vec::new(), index: 0, pending: String::new() }
    }

    fn show(&mut self, choices: &[&str], pending: &str) {
        self.open = true;
        self.choices = choices.iter().map(|s| s.to_string()).collect();
        self.index = 0;
        self.pending = pending.to_string();
    }

    fn close(&mut self) {
        self.open = false;
        self.pending.clear();
    }
}

#[derive(Clone, Debug)]
struct DamageNumber {
    text: String,
    pos: Vec2,
    timer: f32,
}

#[derive(Clone, Debug)]
struct AttackEffect {
    pos: Vec2,
    timer: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InvFocus {
    Inventory,
    Equipment,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MapTransition {
    FadeOut { elapsed: f32, target: usize },
    FadeIn { elapsed: f32 },
}

pub struct Game {
    assets: Assets,
    cli: Cli,
    screen: Screen,
    map_index: usize,
    map: MapData,
    player: Player,
    enemies: EnemyManager,
    items: HashMap<String, Item>,
    inventory: Inventory,
    equipment: Equipment,
    dialogue: Dialogue,
    options: Options,
    scripts: Vec<ScriptRunner>,
    script_flags: HashSet<String>,
    chests_opened: Vec<bool>,
    fake_opened: Vec<bool>,
    switches: Vec<bool>,
    plates: Vec<bool>,
    room_light: Vec<f32>,
    room_light_ready: bool,
    visible_tiles: HashSet<Tile>,
    visible_rooms: Vec<String>,
    damage_numbers: Vec<DamageNumber>,
    attack_effects: Vec<AttackEffect>,
    unsaved: bool,
    active_run: bool,
    can_resume: bool,
    close_prompt: bool,
    new_game_prompt: bool,
    bump_lock: bool,
    death_timer: Option<f32>,
    transition: Option<MapTransition>,
    slot: usize,
    settings_index: usize,
    settings_message: String,
    debug_minimized: bool,
    debug_editor_open: bool,
    debug_edit_index: usize,
    debug_edit_value: Option<String>,
    debug_message: String,
    help_open: bool,
    available_scripts: Vec<String>,
    inventory_open: bool,
    inv_focus: InvFocus,
    inv_index: usize,
    equip_index: usize,
    camera: Vec2,
    freecam: Vec2,
    heart_shake_time: f32,
    rng_state: u64,
    spike_timer: f32,
    enemy_damage_timer: f32,
    fountain_timer: f32,
    door_unlocked: bool,
    slot_menu_message: String,
}

impl Game {
    pub fn new(assets: Assets, cli: Cli) -> Self {
        let items = crate::items::load_items(&assets.src().join("data/items.json"));
        let map = MapData::load(&assets.src().join("maps").join(config::MAP_FILES[cli.level]));
        let mut enemies = EnemyManager::load(&map, &assets);
        enemies.set_disabled(cli.no_enemies);
        let player = Player::new(map.spawn, cli.health_infinite);
        let chest_count = map.objects_named_prefix("chest").len();
        let fake_count = map.objects_named_prefix("fakeChest").len();
        let switch_count = map.objects_named_prefix("switch").len();
        let plate_count = map.objects_named_prefix("pressurePlate").len();
        let room_light = vec![0.0; (map.width * map.height) as usize];
        let mut available_scripts = Vec::new();
        if let Ok(entries) = std::fs::read_dir(assets.src().join("scripts")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) != Some("urpgs") {
                    continue;
                }
                if let Some(name) = path.file_stem().and_then(|value| value.to_str()) {
                    available_scripts.push(name.to_string());
                }
            }
        }
        available_scripts.sort();
        let mut game = Self {
            screen: Screen::Menu,
            map_index: cli.level,
            map,
            player,
            enemies,
            items,
            inventory: Inventory::new(),
            equipment: Equipment::new(),
            dialogue: Dialogue::new(),
            options: Options::new(),
            scripts: Vec::new(),
            script_flags: HashSet::new(),
            chests_opened: vec![false; chest_count],
            fake_opened: vec![false; fake_count],
            switches: vec![false; switch_count],
            plates: vec![false; plate_count],
            room_light,
            room_light_ready: false,
            visible_tiles: HashSet::new(),
            visible_rooms: Vec::new(),
            damage_numbers: Vec::new(),
            attack_effects: Vec::new(),
            unsaved: false,
            active_run: false,
            can_resume: false,
            close_prompt: false,
            new_game_prompt: false,
            bump_lock: false,
            death_timer: None,
            transition: None,
            slot: 1,
            settings_index: 0,
            settings_message: String::new(),
            debug_minimized: false,
            debug_editor_open: false,
            debug_edit_index: 0,
            debug_edit_value: None,
            debug_message: String::new(),
            help_open: false,
            available_scripts,
            inventory_open: false,
            inv_focus: InvFocus::Inventory,
            inv_index: 0,
            equip_index: 0,
            camera: vec2(0.0, 0.0),
            freecam: Vec2::ZERO,
            heart_shake_time: 0.0,
            rng_state: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos() as u64 | 1)
                .unwrap_or(0x9e3779b97f4a7c15),
            spike_timer: 0.0,
            enemy_damage_timer: 0.0,
            fountain_timer: 0.0,
            door_unlocked: false,
            slot_menu_message: String::new(),
            assets,
            cli,
        };
        if game.cli.level_given {
            game.start_game();
        }
        game
    }

    pub async fn run(&mut self) {
        prevent_quit();
        loop {
            let dt = get_frame_time().min(1.0 / 15.0);
            self.update(dt);
            self.draw();
            next_frame().await;
        }
    }

    fn random_index(&mut self, len: usize) -> usize {
        if len <= 1 {
            return 0;
        }
        self.rng_state = self.rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.rng_state >> 33) as usize) % len
    }

    fn room_mode(&self) -> bool {
        self.map.has_rooms() && !self.cli.disable_rooms
    }

    fn start_game(&mut self) {
        self.load_map(self.cli.level);
        self.player.reset(self.map.spawn);
        self.inventory = Inventory::new();
        self.inventory.add("apple", 3, self.items.get("apple").map_or(false, |i| i.stackable));
        self.inventory.add("bronze_sword", 1, false);
        self.inventory.add("stainless_steel_shield", 1, false);
        self.equipment = Equipment::new();
        self.scripts.clear();
        self.chests_opened.iter_mut().for_each(|v| *v = false);
        self.fake_opened.iter_mut().for_each(|v| *v = false);
        self.switches.iter_mut().for_each(|v| *v = false);
        self.plates.iter_mut().for_each(|v| *v = false);
        self.enemies.enemies.clear();
        self.enemies.reset_triggers();
        self.death_timer = None;
        self.damage_numbers.clear();
        self.attack_effects.clear();
        self.enemy_damage_timer = 0.0;
        self.door_unlocked = false;
        self.dialogue.close();
        self.options.close();
        self.unsaved = true;
        self.active_run = true;
        self.can_resume = true;
        self.screen = Screen::Game;
        self.inv_focus = InvFocus::Inventory;
        self.inv_index = 0;
        self.equip_index = 0;
        self.freecam = Vec2::ZERO;
        self.heart_shake_time = 0.0;
        self.camera = self.player.pos;
        if let Some(name) = self.cli.script.clone() {
            self.run_script(&name);
        }
    }

    fn load_map(&mut self, index: usize) {
        self.map_index = index.min(config::MAP_FILES.len() - 1);
        self.map = MapData::load(&self.assets.src().join("maps").join(config::MAP_FILES[self.map_index]));
        log_line(&format!("[map] loaded {}", self.map.name));
        self.enemies = EnemyManager::load(&self.map, &self.assets);
        self.enemies.set_disabled(self.cli.no_enemies);
        self.chests_opened = vec![false; self.map.objects_named_prefix("chest").len()];
        self.fake_opened = vec![false; self.map.objects_named_prefix("fakeChest").len()];
        self.switches = vec![false; self.map.objects_named_prefix("switch").len()];
        self.plates = vec![false; self.map.objects_named_prefix("pressurePlate").len()];
        self.room_light = vec![0.0; (self.map.width * self.map.height) as usize];
        self.room_light_ready = false;
        self.visible_tiles.clear();
        self.visible_rooms.clear();
        self.enemy_damage_timer = 0.0;
        self.fountain_timer = 0.0;
        self.door_unlocked = false;
        self.player.reset(self.map.spawn);
        self.camera = self.player.pos;
        self.freecam = Vec2::ZERO;
        self.update_fog(0.0);
    }

    fn advance_transition(
        transition: &mut Option<MapTransition>,
        dt: f32,
        duration: f32,
    ) -> Option<usize> {
        let duration = duration.max(f32::EPSILON);
        match (*transition)? {
            MapTransition::FadeOut { elapsed, target } => {
                let elapsed = elapsed + dt;
                if elapsed < duration {
                    *transition = Some(MapTransition::FadeOut { elapsed, target });
                    return None;
                }
                *transition = Some(MapTransition::FadeIn { elapsed: 0.0 });
                Some(target)
            }
            MapTransition::FadeIn { elapsed } => {
                let elapsed = elapsed + dt;
                if elapsed < duration {
                    *transition = Some(MapTransition::FadeIn { elapsed });
                } else {
                    *transition = None;
                }
                None
            }
        }
    }

    fn run_script(&mut self, name: &str) {
        if let Some(runner) = ScriptRunner::load(&self.assets, name) {
            self.scripts.push(runner);
        }
    }

    fn script_context(&self) -> ScriptContext {
        let mut items = HashMap::new();
        for id in self.items.keys() {
            items.insert(id.clone(), self.inventory.count(id));
        }
        ScriptContext {
            health: self.player.hp,
            max_health: self.player.max_hp,
            dead: self.player.dead,
            moving: self.player.moving,
            facing: self.player.facing.clone(),
            gold: self.inventory.gold,
            map: std::path::Path::new(&self.map.name)
                .file_stem().unwrap_or_default().to_string_lossy().to_string(),
            flags: self.script_flags.clone(),
            items,
        }
    }

    fn process_script_actions(&mut self, dt: f32) {
        let ctx = self.script_context();
        let dialogue_open = self.dialogue.open;
        let assets = &self.assets;
        let load = |name: &str| ScriptRunner::load(assets, name);
        let mut actions = Vec::new();
        let mut index = 0;
        while index < self.scripts.len() {
            let action = self.scripts[index].update(dt, &ctx, dialogue_open, &load);
            if matches!(action, ScriptAction::None) {
                index += 1;
                continue;
            }
            let end = matches!(action, ScriptAction::End);
            actions.push(action);
            if end {
                self.scripts.remove(index);
            } else {
                index += 1;
            }
        }
        for action in actions {
            self.apply_script_action(action);
        }
    }

    fn apply_script_action(&mut self, action: ScriptAction) {
        match action {
            ScriptAction::Dialogue(text) => self.dialogue.show(&text),
            ScriptAction::Give { id, count } => {
                let Some(item) = self.items.get(&id) else {
                    log_line(&format!("[script] unknown item: {id}"));
                    return;
                };
                let stackable = item.stackable;
                if self.inventory.add(&id, count, stackable) {
                    self.unsaved = true;
                }
            }
            ScriptAction::Take { id, count } => {
                if self.inventory.remove_item(&id, count) {
                    self.unsaved = true;
                }
            }
            ScriptAction::Gold(amount) => {
                self.inventory.gold = (self.inventory.gold + amount).max(0);
                self.unsaved = true;
            }
            ScriptAction::SetGold(amount) => {
                self.inventory.gold = amount.max(0);
                self.unsaved = true;
            }
            ScriptAction::Heal(amount) => {
                self.player.heal(amount);
                self.unsaved = true;
            }
            ScriptAction::Damage(amount) => {
                let dealt = self.player.take_damage(amount, 0);
                if dealt > 0 {
                    self.spawn_damage_number(dealt);
                }
                self.unsaved = true;
            }
            ScriptAction::SetHealth(amount) => {
                self.player.hp = amount.clamp(0, self.player.max_hp);
                self.unsaved = true;
            }
            ScriptAction::Spawn { id, col, row } => {
                let pos = match (col, row) {
                    (Some(c), Some(r)) => vec2(
                        c as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                        r as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                    ),
                    _ => vec2(self.player.pos.x + TILE_SIZE, self.player.pos.y),
                };
                self.enemies.spawn(&id, pos);
            }
            ScriptAction::Flag(name) => {
                self.script_flags.insert(name);
                self.unsaved = true;
            }
            ScriptAction::Unflag(name) => {
                self.script_flags.remove(&name);
                self.unsaved = true;
            }
            ScriptAction::End | ScriptAction::None => {}
        }
    }

    fn spawn_damage_number(&mut self, amount: i32) {
        self.damage_numbers.push(DamageNumber {
            text: format!("-{amount}"),
            pos: self.player.pos + vec2(0.0, -20.0),
            timer: crate::config::get().damage_number_lifetime,
        });
    }

    fn blocked_tiles(&self) -> HashSet<Tile> {
        blocked_tiles_for(&self.map, self.door_unlocked)
    }

    fn update(&mut self, dt: f32) {
        if self.new_game_prompt {
            let pressed: Vec<KeyCode> = get_keys_pressed().into_iter().collect();
            if pressed.contains(&KeyCode::Y)
                || pressed.contains(&KeyCode::Enter)
                || pressed.contains(&KeyCode::Space)
            {
                self.new_game_prompt = false;
                self.start_game();
            } else if pressed.contains(&KeyCode::N) || pressed.contains(&KeyCode::Escape) {
                self.new_game_prompt = false;
            }
            return;
        }
        if is_quit_requested() && self.screen == Screen::Game && self.unsaved {
            self.close_prompt = true;
        }
        if is_quit_requested() && !self.close_prompt {
            std::process::exit(0);
        }
        if self.close_prompt {
            let pressed: Vec<KeyCode> = get_keys_pressed().into_iter().collect();
            if pressed.contains(&KeyCode::Y)
                || pressed.contains(&KeyCode::Enter)
                || pressed.contains(&KeyCode::Space)
            {
                std::process::exit(0);
            }
            if pressed.contains(&KeyCode::N) || pressed.contains(&KeyCode::Escape) {
                self.close_prompt = false;
            }
            return;
        }

        match self.screen {
            Screen::Menu | Screen::SlotLoad | Screen::SlotSave | Screen::Settings | Screen::Won | Screen::Dead => {
                if self.screen == Screen::Dead {
                    if let Some(timer) = self.death_timer.as_mut() {
                        *timer += dt;
                    }
                }
                let pressed: Vec<KeyCode> = get_keys_pressed().into_iter().collect();
                if (self.screen == Screen::Won || self.screen == Screen::Dead)
                    && (pressed.contains(&KeyCode::Enter)
                        || pressed.contains(&KeyCode::Escape)
                        || pressed.contains(&KeyCode::Space))
                {
                    self.screen = Screen::Menu;
                    self.can_resume = false;
                }
                return;
            }
            Screen::Game => {}
        }

        self.update_fog(dt);

        if self.transition.is_some() {
            let duration = crate::config::get().transition_duration;
            if let Some(target) = Self::advance_transition(&mut self.transition, dt, duration) {
                self.load_map(target);
                self.dialogue.show("The door opens. You go deeper into the dungeon.");
            }
            return;
        }

        let pressed: Vec<KeyCode> = get_keys_pressed().into_iter().collect();
        if self.handle_debug_keys(&pressed) {
            return;
        }
        if pressed.contains(&KeyCode::F1) {
            self.help_open = !self.help_open;
            return;
        }
        if self.help_open {
            if pressed.contains(&KeyCode::Escape) || pressed.contains(&KeyCode::Enter) {
                self.help_open = false;
            }
            return;
        }
        if self.dialogue.open || self.options.open {
            self.update_box(&pressed, dt);
            return;
        }

        let down: Vec<KeyCode> = get_keys_down().into_iter().collect();
        let hearts_left = (self.player.hp + crate::config::get().health_per_heart - 1)
            / crate::config::get().health_per_heart;
        if self.player.hp > 0 && hearts_left <= 3 {
            self.heart_shake_time += dt;
        } else {
            self.heart_shake_time = 0.0;
        }
        self.handle_game_keys(&pressed);
        if self.inventory_open {
            return;
        }
        self.process_script_actions(dt);

        if self.cli.freecam {
            let mut delta = Vec2::ZERO;
            if down.contains(&KeyCode::Left) { delta.x -= 300.0 * dt; }
            if down.contains(&KeyCode::Right) { delta.x += 300.0 * dt; }
            if down.contains(&KeyCode::Up) { delta.y -= 300.0 * dt; }
            if down.contains(&KeyCode::Down) { delta.y += 300.0 * dt; }
            self.freecam += delta;
        }

        if self.player.dead {
            if self.death_timer.is_none() {
                self.death_timer = Some(0.0);
            }
            if let Some(timer) = self.death_timer.as_mut() {
                *timer += dt;
                if *timer >= crate::config::get().death_screen_delay {
                    self.screen = Screen::Dead;
                    self.active_run = false;
                }
            }
            self.player.update_death_animation(dt);
            return;
        }

        let blocked = self.blocked_tiles();
        let move_dir = if down.contains(&KeyCode::W) {
            Some((0, -1, "up"))
        } else if down.contains(&KeyCode::S) {
            Some((0, 1, "down"))
        } else if down.contains(&KeyCode::A) {
            Some((-1, 0, "left"))
        } else if down.contains(&KeyCode::D) {
            Some((1, 0, "right"))
        } else {
            None
        };
        if move_dir.is_none() {
            self.bump_lock = false;
        }
        if !self.cli.freecam
            && !self.player.moving
            && !self.dialogue.open
            && !self.options.open
            && !self.inventory_open
            && !self.bump_lock
        {
            if let Some((dx, dy, facing)) = move_dir {
                self.player.set_facing(facing);
                let from = self.map.tile_of(self.player.pos);
                let next = Tile(from.0 + dx, from.1 + dy);
                if !self.map.in_bounds(next) || self.map.is_solid(next) || blocked.contains(&next) {
                    self.bump_interact(next);
                    if self.dialogue.open || self.options.open {
                        self.bump_lock = true;
                    }
                }
            }
        }
        self.enemies.update(dt);
        let moved = self.player.update(dt, &self.map, &blocked, &down, &pressed);
        if !self.cli.freecam {
            self.camera = self.player.pos;
        }
        self.enemies.update_triggers(dt, self.map.tile_of(self.player.target), &self.map);

        for effect in self.attack_effects.iter_mut() {
            effect.timer -= dt;
        }
        self.attack_effects.retain(|e| e.timer > 0.0);
        for number in self.damage_numbers.iter_mut() {
            number.timer -= dt;
            number.pos.y -= crate::config::get().damage_number_rise * dt;
        }
        self.damage_numbers.retain(|n| n.timer > 0.0);

        if moved {
            log_line(&format!(
                "Moved: ({:.0}, {:.0}) | Tile: ({}, {}) | Facing: {} | HP: {}/{}",
                self.player.pos.x, self.player.pos.y,
                self.map.tile_of(self.player.pos).0, self.map.tile_of(self.player.pos).1,
                self.player.facing, self.player.hp, self.player.max_hp
            ));
            self.log_rooms();
            self.enemies.step_toward_player(self.player.pos, &self.map);
            self.enemy_damage(dt);
        } else {
            self.enemy_damage(dt);
        }

        self.fountain_timer = (self.fountain_timer - dt).max(0.0);
        self.spike_timer -= dt;
        if self.map.spikes.contains(&self.map.tile_of(self.player.pos)) && self.spike_timer <= 0.0 {
            let (_, defense) = self.equipment.combat_stats(&self.items);
            let dealt = self.player.take_damage(crate::config::get().spike_damage, defense);
            if dealt > 0 {
                self.spawn_damage_number(dealt);
            }
            self.spike_timer = crate::config::get().spike_cooldown;
        } else if !self.map.spikes.contains(&self.map.tile_of(self.player.pos)) {
            self.spike_timer = 0.0;
        }
    }

    fn enemy_damage(&mut self, dt: f32) {
        let Some((damage, damage_speed)) = self.enemies.contact_attack(self.player.pos) else {
            self.enemy_damage_timer = 0.0;
            return;
        };
        if self.enemy_damage_timer > 0.0 {
            self.enemy_damage_timer = (self.enemy_damage_timer - dt).max(0.0);
            return;
        }
        if damage > 0 {
            let (_, defense) = self.equipment.combat_stats(&self.items);
            let dealt = self.player.take_damage(damage, defense);
            if dealt > 0 {
                self.spawn_damage_number(dealt);
            }
        }
        self.enemy_damage_timer = if damage_speed > 0.0 {
            damage_speed / 10.0
        } else {
            crate::config::get().enemy_damage_cooldown
        };
    }

    fn log_rooms(&self) {
        if !self.room_mode() {
            return;
        }
        log_line(&format!(
            "[rooms] {}: {} | {} visible tiles",
            self.map.name,
            if self.visible_rooms.is_empty() { "none".into() } else { self.visible_rooms.join(", ") },
            self.visible_tiles.len()
        ));
    }

    fn update_fog(&mut self, dt: f32) {
        if !self.room_mode() {
            return;
        }
        let tile = self.map.tile_of(self.player.pos);
        self.visible_rooms = self.map.rooms_at(tile);
        self.visible_tiles = self.map.visible_tiles(tile);
        let target = self.room_light_targets();
        if !self.room_light_ready {
            self.room_light = target;
            self.room_light_ready = true;
            return;
        }
        let mut difference = 0.0f64;
        for (current, target) in self.room_light.iter().zip(target.iter()) {
            difference += (*target - *current).abs() as f64;
        }
        let mean = difference / target.len().max(1) as f64;
        if mean <= 1e-4 {
            return;
        }
        let rate = crate::config::get().room_light_speed * (1.0 + crate::config::get().room_light_boost * mean as f32);
        let factor = 1.0 - (-rate * dt).exp();
        for (current, target) in self.room_light.iter_mut().zip(target.iter()) {
            *current += (*target - *current) * factor;
        }
    }

    fn room_light_targets(&self) -> Vec<f32> {
        use std::collections::VecDeque;

        let cols = self.map.width as usize;
        let rows = self.map.height as usize;
        let infinite = (cols * rows + 1) as i32;
        let mut dist = vec![infinite; cols * rows];
        let mut queue = VecDeque::new();
        for tile in &self.visible_tiles {
            if tile.0 >= 0 && tile.1 >= 0 && (tile.0 as usize) < cols && (tile.1 as usize) < rows {
                let index = tile.1 as usize * cols + tile.0 as usize;
                if dist[index] != 0 {
                    dist[index] = 0;
                    queue.push_back((tile.0 as usize, tile.1 as usize));
                }
            }
        }
        while let Some((col, row)) = queue.pop_front() {
            let next_distance = dist[row * cols + col] + 1;
            for dc in -1i32..=1 {
                for dr in -1i32..=1 {
                    if dc == 0 && dr == 0 {
                        continue;
                    }
                    let nc = col as i32 + dc;
                    let nr = row as i32 + dr;
                    if nc < 0 || nr < 0 || nc >= cols as i32 || nr >= rows as i32 {
                        continue;
                    }
                    let index = nr as usize * cols + nc as usize;
                    if dist[index] > next_distance {
                        dist[index] = next_distance;
                        queue.push_back((nc as usize, nr as usize));
                    }
                }
            }
        }
        let mut target = vec![0.0; cols * rows];
        for (index, distance) in dist.iter().enumerate() {
            let level = if *distance == 0 {
                1.0
            } else if *distance < crate::config::get().room_fade_tiles {
                (1.0 - *distance as f32 / crate::config::get().room_fade_tiles as f32).max(0.0)
            } else {
                0.0
            };
            target[index] = level * (1.0 - crate::config::get().room_dim);
        }
        target
    }

    fn update_box(&mut self, pressed: &[KeyCode], dt: f32) {
        self.dialogue.update(dt);
        if pressed.contains(&KeyCode::Z) {
            self.dialogue.fast = true;
            return;
        }
        if pressed.contains(&KeyCode::Escape) {
            self.dialogue.close();
            self.options.close();
            return;
        }
        if self.options.open {
            if pressed.contains(&KeyCode::Left) || pressed.contains(&KeyCode::Up) {
                let count = self.options.choices.len().max(1);
                self.options.index = (self.options.index + count - 1) % count;
            }
            if pressed.contains(&KeyCode::Right) || pressed.contains(&KeyCode::Down) {
                let count = self.options.choices.len().max(1);
                self.options.index = (self.options.index + 1) % count;
            }
            if pressed.contains(&KeyCode::Enter) || pressed.contains(&KeyCode::Space) {
                if !self.dialogue.complete() {
                    self.dialogue.finish();
                    return;
                }
                let choice = self.options.choices.get(self.options.index).cloned().unwrap_or_default();
                let pending = self.options.pending.clone();
                self.options.close();
                self.dialogue.close();
                self.resolve_option(&pending, &choice);
            }
            return;
        }
        if pressed.contains(&KeyCode::Enter) || pressed.contains(&KeyCode::Space) {
            if self.dialogue.complete() {
                self.dialogue.close();
            } else {
                self.dialogue.finish();
            }
        }
    }

    fn show_options(&mut self, message: &str, choices: &[&str], pending: &str) {
        self.dialogue.show(message);
        self.options.show(choices, pending);
    }

    fn open_debug_editor(&mut self) {
        let keys = config::keys();
        self.debug_edit_index = config::modified()
            .first()
            .and_then(|(key, _, _)| keys.iter().position(|value| value == key))
            .unwrap_or(0);
        self.debug_editor_open = true;
        self.debug_message.clear();
    }

    fn handle_debug_keys(&mut self, pressed: &[KeyCode]) -> bool {
        if !self.cli.debug {
            return false;
        }
        if self.debug_edit_value.is_none() {
            while get_char_pressed().is_some() {}
        }
        if let Some(buffer) = self.debug_edit_value.as_mut() {
            if let Some(ch) = get_char_pressed() {
                buffer.push(ch);
            }
            if pressed.contains(&KeyCode::Backspace) {
                buffer.pop();
            }
            if pressed.contains(&KeyCode::Escape) {
                self.debug_edit_value = None;
                self.debug_message.clear();
                return true;
            }
            if pressed.contains(&KeyCode::Enter) {
                let keys = config::keys();
                if keys.is_empty() {
                    self.debug_edit_value = None;
                    return true;
                }
                let key = keys[self.debug_edit_index.min(keys.len() - 1)].clone();
                let value = self.debug_edit_value.take().unwrap_or_default();
                if config::set_value(&key, &value) {
                    self.debug_message = format!("{key} = {value}");
                    self.unsaved = true;
                    log_line(&format!("[debug] {key}: {value}"));
                } else {
                    self.debug_message = format!("invalid value for {key}");
                }
                return true;
            }
            return true;
        }
        if pressed.contains(&KeyCode::M) {
            self.debug_minimized = !self.debug_minimized;
            self.debug_editor_open = false;
            self.debug_message.clear();
            return true;
        }
        if self.debug_minimized {
            return false;
        }
        if self.debug_editor_open {
            if pressed.contains(&KeyCode::Escape) {
                self.debug_editor_open = false;
                self.debug_message.clear();
                return true;
            }
            let count = config::keys().len();
            if count == 0 {
                return true;
            }
            if pressed.contains(&KeyCode::Up) || pressed.contains(&KeyCode::W) {
                self.debug_edit_index = (self.debug_edit_index + count - 1) % count;
            }
            if pressed.contains(&KeyCode::Down) || pressed.contains(&KeyCode::S) {
                self.debug_edit_index = (self.debug_edit_index + 1) % count;
            }
            if pressed.contains(&KeyCode::Home) {
                self.debug_edit_index = 0;
            }
            if pressed.contains(&KeyCode::End) {
                self.debug_edit_index = count - 1;
            }
            if pressed.contains(&KeyCode::A) {
                config::reset_all();
                self.debug_message = "all config values reset to original".into();
                self.unsaved = true;
                log_line("[debug] all config values reset to original");
                return true;
            }
            if pressed.contains(&KeyCode::R) {
                let keys = config::keys();
                let key = keys[self.debug_edit_index.min(keys.len() - 1)].clone();
                if config::reset_value(&key) {
                    self.debug_message = format!("{key} reset to original");
                    self.unsaved = true;
                    log_line(&format!("[debug] {key} reset to original"));
                } else {
                    self.debug_message = format!("cannot reset {key}");
                }
                return true;
            }
            if pressed.contains(&KeyCode::Enter) || pressed.contains(&KeyCode::E) {
                let keys = config::keys();
                let key = keys[self.debug_edit_index.min(keys.len() - 1)].clone();
                self.debug_edit_value = Some(config::value(&key).unwrap_or_default());
                self.debug_message = format!("editing {key}");
                return true;
            }
            return true;
        }
        if pressed.contains(&KeyCode::E) {
            self.open_debug_editor();
            return true;
        }
        false
    }

    fn handle_game_keys(&mut self, pressed: &[KeyCode]) {
        if self.inventory_open {
            if pressed.contains(&KeyCode::I) || pressed.contains(&KeyCode::Escape) {
                self.inventory_open = false;
                return;
            }
            if pressed.contains(&KeyCode::Tab) {
                self.inv_focus = match self.inv_focus {
                    InvFocus::Inventory => InvFocus::Equipment,
                    InvFocus::Equipment => InvFocus::Inventory,
                };
            }
            self.handle_inventory_keys(pressed);
            return;
        }
        for key in pressed {
            match key {
                KeyCode::Z => {
                    if !self.player.moving && !self.player.dead {
                        let pos = self.player.attack_position();
                        self.attack_effects.push(AttackEffect { pos, timer: crate::config::get().attack_effect_duration });
                        self.enemies.attack_at(pos);
                    }
                }
                KeyCode::E | KeyCode::Enter => self.interact(),
                KeyCode::I => {
                    self.screen = Screen::Game;
                    self.inventory_open = true;
                }
                KeyCode::Escape => {
                    if self.active_run && !self.player.dead {
                        self.can_resume = true;
                    }
                    self.screen = Screen::Menu;
                }
                KeyCode::H => {
                    self.enemies.spawn_near_player(self.player.pos, &self.map);
                }
                _ => {}
            }
        }
    }

    fn resolve_option(&mut self, pending: &str, choice: &str) {
        if pending == "door" {
            if choice == "give" {
                self.open_door();
            } else {
                self.dialogue.show("You decided not to give the key.");
            }
            return;
        }
        if pending == "fountain" {
            if choice == "yes" {
                if self.fountain_timer <= 0.0 {
                    self.player.heal(crate::config::get().fountain_heal);
                    self.fountain_timer = crate::config::get().fountain_cooldown;
                    self.dialogue.show("You drank from the fountain and got refreshed! (2 hearts healed)");
                } else {
                    self.dialogue.show("Don't drink too much water!");
                }
            } else {
                self.dialogue.show("You chose not to drink.");
            }
            return;
        }
        if let Some(index) = pending.strip_prefix("switch:").and_then(|v| v.parse::<usize>().ok()) {
            if choice != "yes" {
                return;
            }
            if let Some(value) = self.switches.get_mut(index) {
                *value = !*value;
                self.dialogue.show(if *value {
                    "The switch is now on."
                } else {
                    "The switch is now off."
                });
            }
        } else if let Some(index) = pending.strip_prefix("pressure_plate:").and_then(|v| v.parse::<usize>().ok()) {
            if choice == "yes" {
                if let Some(value) = self.plates.get_mut(index) {
                    *value = true;
                }
            }
        }
    }

    fn interact(&mut self) {
        if self.player.dead || self.player.moving {
            return;
        }
        if let Some(index) = self.map.object_in_front(self.player.pos, &self.player.facing, "chest") {
            self.open_chest(index);
            return;
        }
        if let Some(index) = self.map.object_in_front(self.player.pos, &self.player.facing, "fakeChest") {
            self.open_fake_chest(index);
            return;
        }
        let _handled = self.interact_switch_in_front()
            || self.interact_plate_in_front()
            || self.interact_script_in_front()
            || self.interact_door_in_front();
    }

    fn bump_interact(&mut self, next: Tile) {
        if self.player.dead || self.player.moving {
            return;
        }
        let pos = self.player.pos;
        let fountain = self.map.object_index_at_tile("fountain", next);
        if let Some(index) = fountain {
            self.open_fountain(index);
            return;
        }
        if self.interact_switch_in_front() || self.interact_plate_in_front() {
            return;
        }
        let chest = self
            .map
            .object_index_at_tile("chest", next)
            .or_else(|| self.map.object_in_front(pos, &self.player.facing, "chest"));
        if let Some(index) = chest {
            self.open_chest(index);
            return;
        }
        let fake_chest = self
            .map
            .object_index_at_tile("fakeChest", next)
            .or_else(|| self.map.object_in_front(pos, &self.player.facing, "fakeChest"));
        if let Some(index) = fake_chest {
            self.open_fake_chest(index);
            return;
        }
        let _handled = self.interact_script_in_front() || self.interact_door_in_front();
    }

    fn interact_switch_in_front(&mut self) -> bool {
        let Some(index) = self.map.object_in_front(self.player.pos, &self.player.facing, "switch") else {
            return false;
        };
        let message = if self.switches.get(index).copied().unwrap_or(false) {
            "The switch is currently on, flip it?"
        } else {
            "The switch is currently off, flip it?"
        };
        self.show_options(message, &["yes", "no"], &format!("switch:{index}"));
        true
    }

    fn interact_plate_in_front(&mut self) -> bool {
        let Some(index) = self.map.object_in_front(self.player.pos, &self.player.facing, "pressurePlate") else {
            return false;
        };
        if self.plates.get(index).copied().unwrap_or(false) {
            return false;
        }
        self.show_options(
            "Step on the pressure plate?",
            &["yes", "no"],
            &format!("pressure_plate:{index}"),
        );
        true
    }

    fn interact_script_in_front(&mut self) -> bool {
        let name = self.map.objects_named_prefix("script:").into_iter()
            .find(|obj| self.object_is_in_front(obj))
            .map(|obj| obj.name.trim_start_matches("script:").to_string());
        if let Some(name) = name {
            self.run_script(&name);
            return true;
        }
        false
    }

    fn interact_door_in_front(&mut self) -> bool {
        let Some(door) = self.map.object_named("door") else {
            return false;
        };
        if !self.object_is_in_front(door) {
            return false;
        }
        let message = if self.map_index + 1 >= config::MAP_FILES.len() {
            "The final door needs a key. Give the key?"
        } else {
            "The door is locked. Give the key?"
        };
        self.show_options(message, &["give", "not give"], "door");
        true
    }

    fn open_fountain(&mut self, _index: usize) {
        self.bump_lock = true;
        if self.fountain_timer > 0.0 {
            self.dialogue.show("Don't drink too much water!");
        } else {
            self.show_options(
                "A regenerating fountain, drink from here?",
                &["yes", "no"],
                "fountain",
            );
        }
    }

    fn object_is_in_front(&self, obj: &crate::map::MapObject) -> bool {
        (vec2(obj.x, obj.y) - self.player.attack_position()).length() <= crate::config::get().interaction_distance
    }

    fn open_chest(&mut self, index: usize) {
        if self.chests_opened.get(index).copied().unwrap_or(false) {
            self.dialogue.show("This chest has already been opened.");
            return;
        }
        let rewards = ["apple", "iron_sword", "rusty_iron_sword", "leather_armour", "titanium_sword"];
        let unopened = self.chests_opened.iter().filter(|opened| !**opened).count();
        let reward = if unopened == 1 {
            "key"
        } else {
            let pick = self.random_index(rewards.len());
            rewards[pick]
        };
        let stackable = self.items.get(reward).map_or(false, |i| i.stackable);
        if !self.inventory.add(reward, 1, stackable) {
            self.dialogue.show("Your inventory is full. Make room before opening this chest.");
            return;
        }
        if index < self.chests_opened.len() {
            self.chests_opened[index] = true;
        }
        let name = self.items.get(reward).map(|i| i.name.as_str()).unwrap_or(reward);
        if reward == "key" {
            self.dialogue.show("The last chest contained the key to the door!");
        } else {
            self.dialogue.show(&format!("The chest contained {name}."));
        }
        self.unsaved = true;
    }

    fn open_fake_chest(&mut self, index: usize) {
        let messages = [
            "It's a fake chest! Nothing inside but your own greed.",
            "Empty. The chest seems to be laughing at you.",
            "You fell for it. There is nothing here but dust.",
            "This chest was never real. Your greed was, though.",
            "The lid creaks open to reveal... disappointment.",
        ];
        let message = if self.fake_opened.get(index).copied().unwrap_or(false) {
            "Still empty. Still greedy."
        } else {
            let pick = self.random_index(messages.len());
            messages[pick]
        };
        if index < self.fake_opened.len() {
            self.fake_opened[index] = true;
        }
        self.dialogue.show(message);
    }

    fn open_door(&mut self) {
        if !self.inventory.remove_item("key", 1) {
            self.dialogue.show("You do not have a key.");
            return;
        }
        self.door_unlocked = true;
        if self.map_index + 1 >= config::MAP_FILES.len() {
            self.screen = Screen::Won;
            self.unsaved = false;
            self.active_run = false;
            return;
        }
        self.transition = Some(MapTransition::FadeOut {
            elapsed: 0.0,
            target: self.map_index + 1,
        });
    }

    fn handle_inventory_keys(&mut self, pressed: &[KeyCode]) {
        let columns = crate::config::get().columns as i32;
        let slots = self.inventory.slots.len().max(1) as i32;
        match self.inv_focus {
            InvFocus::Inventory => {
                if pressed.contains(&KeyCode::Left) || pressed.contains(&KeyCode::A) {
                    if self.inv_index % columns as usize == 0 {
                        self.inv_focus = InvFocus::Equipment;
                    } else {
                        self.inv_index = (self.inv_index as i32 - 1).rem_euclid(slots) as usize;
                    }
                }
                if pressed.contains(&KeyCode::Right) || pressed.contains(&KeyCode::D) {
                    self.inv_index = (self.inv_index as i32 + 1).rem_euclid(slots) as usize;
                }
                if pressed.contains(&KeyCode::Up) || pressed.contains(&KeyCode::W) {
                    self.inv_index = (self.inv_index as i32 - columns).rem_euclid(slots) as usize;
                }
                if pressed.contains(&KeyCode::Down) || pressed.contains(&KeyCode::S) {
                    self.inv_index = (self.inv_index as i32 + columns).rem_euclid(slots) as usize;
                }
                if pressed.contains(&KeyCode::Enter) || pressed.contains(&KeyCode::Space) {
                    self.inventory_confirm();
                }
                if pressed.contains(&KeyCode::R) {
                    self.inventory_sell();
                }
            }
            InvFocus::Equipment => {
                if pressed.contains(&KeyCode::Up) || pressed.contains(&KeyCode::W) {
                    self.equip_index = (self.equip_index + 2) % config::EQUIPMENT_SLOTS.len();
                }
                if pressed.contains(&KeyCode::Down) || pressed.contains(&KeyCode::S) {
                    self.equip_index = (self.equip_index + 1) % config::EQUIPMENT_SLOTS.len();
                }
                if pressed.contains(&KeyCode::Right) || pressed.contains(&KeyCode::D) {
                    self.inv_focus = InvFocus::Inventory;
                }
                if pressed.contains(&KeyCode::Enter) || pressed.contains(&KeyCode::Space) {
                    self.equipment_unequip();
                }
            }
        }
    }

    fn inventory_confirm(&mut self) {
        let Some(stack) = self.inventory.slots.get(self.inv_index).cloned().flatten() else { return };
        let Some(item) = self.items.get(&stack.item_id).cloned() else { return };
        if item.item_type == "consumable" && item.heal > 0 {
            self.inventory.remove_at(self.inv_index, 1);
            self.player.heal(item.heal);
            self.unsaved = true;
        } else if let Some(slot) = item.slot.clone() {
            self.inventory.remove_at(self.inv_index, 1);
            let previous = self.equipment.equip(&slot, &stack.item_id);
            if let Some(previous) = previous {
                let stackable = self.items.get(&previous).map_or(false, |i| i.stackable);
                self.inventory.add(&previous, 1, stackable);
            }
            self.unsaved = true;
        }
    }

    fn inventory_sell(&mut self) {
        let Some(stack) = self.inventory.slots.get(self.inv_index).cloned().flatten() else { return };
        let Some(item) = self.items.get(&stack.item_id).cloned() else { return };
        if !item.removeable {
            return;
        }
        self.inventory.remove_at(self.inv_index, 1);
        self.inventory.gold += item.price;
        self.unsaved = true;
    }

    fn equipment_unequip(&mut self) {
        let slot = config::EQUIPMENT_SLOTS[self.equip_index];
        let Some(item_id) = self.equipment.unequip(slot) else { return };
        let stackable = self.items.get(&item_id).map_or(false, |i| i.stackable);
        if !self.inventory.add(&item_id, 1, stackable) {
            self.equipment.equip(slot, &item_id);
            return;
        }
        self.unsaved = true;
    }

    fn save_game(&mut self) {
        let state = self.build_save_state();
        save::save(&self.assets, self.slot, &state);
        self.unsaved = false;
        self.slot_menu_message = format!("Game saved to slot {}.", self.slot);
        self.screen = Screen::Menu;
    }

    fn load_game(&mut self) {
        let Some(state) = save::load(&self.assets, self.slot) else {
            self.slot_menu_message = format!("No save in slot {}.", self.slot);
            return;
        };
        self.apply_save_state(state);
        self.unsaved = false;
        self.active_run = true;
        self.can_resume = true;
        self.slot_menu_message = format!("Loaded slot {}.", self.slot);
    }

    fn build_save_state(&self) -> SaveState {
        SaveState {
            timestamp: save::timestamp(),
            map: self.map.name.clone(),
            player: save::PlayerSave {
                x: self.player.pos.x,
                y: self.player.pos.y,
                hp: self.player.hp,
                direction: self.player.direction.clone(),
            },
            inventory: save::InventorySave {
                slots: self.inventory.slots.iter().map(|s| s.as_ref().map(|stack| save::StackSave {
                    item_id: stack.item_id.clone(), count: stack.count,
                })).collect(),
            },
            equipment: save::EquipmentSave { slots: self.equipment.slots.clone() },
            gold: self.inventory.gold,
            chests: save::ChestSave {
                opened: self.chests_opened.iter().enumerate().filter_map(|(i, v)| v.then_some(i)).collect(),
                fake_opened: self.fake_opened.iter().enumerate().filter_map(|(i, v)| v.then_some(i)).collect(),
            },
            enemies: save::EnemySave {
                enemies: self.enemies.enemies.iter().filter(|e| !e.dying).map(|e| save::EnemyEntry {
                    kind: e.mob.clone(), x: e.pos.x, y: e.pos.y,
                }).collect(),
                spawned: self.enemies.spawned.iter().enumerate().filter_map(|(i, v)| v.then_some(i)).collect(),
            },
            map_objects: save::MapObjectSave {
                switches: self.switches.clone(),
                pressure_plates: self.plates.clone(),
            },
            script_flags: self.script_flags.iter().cloned().collect(),
            door_unlocked: self.door_unlocked || self.screen == Screen::Won,
            config: config::snapshot(),
        }
    }

    fn apply_save_state(&mut self, state: SaveState) {
        config::restore(&state.config);
        let map_name = state.map;
        let map_index = config::MAP_FILES.iter().position(|m| *m == map_name).unwrap_or(0);
        self.load_map(map_index);
        self.player.pos = vec2(state.player.x, state.player.y);
        self.player.target = self.player.pos;
        self.player.hp = state.player.hp;
        self.player.direction = state.player.direction;
        self.player.facing = self.player.direction.clone();
        self.player.dead = self.player.hp <= 0;
        self.player.moving = false;
        self.inventory.slots = state.inventory.slots.into_iter()
            .map(|s| s.map(|stack| Stack { item_id: stack.item_id, count: stack.count }))
            .collect();
        self.inventory.slots.resize(12, None);
        self.inventory.gold = state.gold;
        self.equipment.slots = state.equipment.slots;
        for index in 0..self.chests_opened.len() {
            self.chests_opened[index] = state.chests.opened.contains(&index);
        }
        for index in 0..self.fake_opened.len() {
            self.fake_opened[index] = state.chests.fake_opened.contains(&index);
        }
        self.switches = state.map_objects.switches;
        self.switches.resize(self.map.objects_named_prefix("switch").len(), false);
        self.plates = state.map_objects.pressure_plates;
        self.plates.resize(self.map.objects_named_prefix("pressurePlate").len(), false);
        self.enemies.enemies.clear();
        for enemy in state.enemies.enemies {
            self.enemies.spawn(&enemy.kind, vec2(enemy.x, enemy.y));
        }
        for index in state.enemies.spawned {
            if let Some(value) = self.enemies.spawned.get_mut(index) {
                *value = true;
            }
        }
        self.enemies.set_disabled(self.cli.no_enemies);
        self.script_flags = state.script_flags.into_iter().collect();
        self.scripts.clear();
        self.death_timer = None;
        self.inv_focus = InvFocus::Inventory;
        self.inv_index = 0;
        self.equip_index = 0;
        self.freecam = Vec2::ZERO;
        self.heart_shake_time = 0.0;
        self.camera = self.player.pos;
        self.door_unlocked = state.door_unlocked;
        self.screen = if state.door_unlocked { Screen::Won } else { Screen::Game };
    }

    fn draw(&mut self) {
        clear_background(config::MENU_BACKGROUND);
        match self.screen {
            Screen::Menu => self.draw_menu(),
            Screen::SlotLoad => self.draw_slot_menu("LOAD GAME"),
            Screen::SlotSave => self.draw_slot_menu("SAVE GAME"),
            Screen::Settings => self.draw_settings(),
            Screen::Won => self.draw_end_screen("YOU ESCAPED!", "The key turned, and the dungeon door opened.", "Press ENTER or ESC to return to the menu."),
            Screen::Dead => self.draw_end_screen(&crate::config::get().death_title, &crate::config::get().death_message, &crate::config::get().death_hint),
            Screen::Game => self.draw_game(),
        }
        if self.close_prompt {
            self.draw_close_prompt();
        }
        if self.new_game_prompt {
            self.draw_new_game_prompt();
        }
        if self.cli.debug {
            self.draw_debug();
        }
        if self.help_open {
            self.draw_help();
        }
    }

    fn draw_debug(&self) {
        let accent = Color::new(0.35, 1.0, 0.5, 1.0);
        let background = Color::new(0.01, 0.06, 0.05, 0.94);
        let size = 12;
        let pad = 10.0;
        let line_h = 18.0;
        let right = screen_width() - 12.0;
        let top = 12.0;

        if self.debug_minimized {
            let label = "DEBUG  [M] expand";
            let width = measure_text(label, Some(&self.assets.font), 14, 1.0).width + pad * 2.0;
            let height = 34.0;
            draw_rectangle(right - width, top, width, height, background);
            draw_rectangle_lines(right - width, top, width, height, 2.0, accent);
            draw_text_at(label, right - pad, top + height / 2.0, 14, accent,
                &self.assets.font, 1.0, 0.5);
            return;
        }

        let modified = config::modified();
        let mut lines: Vec<String> = vec!["DEBUG  [M] minimize  [E] edit".into()];
        if modified.is_empty() {
            lines.push("(no modified config values)".into());
        } else {
            lines.extend(modified.into_iter().map(|(key, current, original)| {
                format!("{key}: {current} (Original {original})")
            }));
        }
        let width = lines
            .iter()
            .map(|line| measure_text(line, Some(&self.assets.font), size, 1.0).width)
            .fold(0.0, f32::max) + pad * 2.0;
        let height = lines.len() as f32 * line_h + pad * 2.0;
        draw_rectangle(right - width, top, width, height, background);
        draw_rectangle_lines(right - width, top, width, height, 2.0, accent);
        for (index, line) in lines.iter().enumerate() {
            let color = if index == 0 { accent } else { TEXT_COLOR };
            draw_text_at(line, right - pad, top + pad + line_h * (index as f32 + 0.5),
                size, color, &self.assets.font, 1.0, 0.5);
        }
        self.draw_config_editor();
    }

    fn draw_config_editor(&self) {
        if !self.debug_editor_open {
            return;
        }
        let keys = config::keys();
        let size = 12;
        let pad = 14.0;
        let line_h = 18.0;
        let per_page = 12usize;
        let box_w = (screen_width() - 80.0).min(720.0);
        let box_h = 150.0 + per_page as f32 * line_h;
        let left = (screen_width() - box_w) / 2.0;
        let top = (screen_height() - box_h) / 2.0;
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), INV_DIM_COLOR);
        draw_rectangle(left, top, box_w, box_h, PANEL_COLOR);
        draw_rectangle_lines(left, top, box_w, box_h, 2.0, INV_ACCENT_COLOR);
        draw_text_at("CONFIG EDITOR", left + pad, top + 28.0, 18, INV_ACCENT_COLOR,
            &self.assets.font, 0.0, 0.5);
        draw_text_at("UP/DOWN select   ENTER edit   R reset   A reset all   ESC close", left + pad, top + 52.0,
            11, MENU_MUTED_TEXT, &self.assets.font, 0.0, 0.5);

        let count = keys.len();
        if count > 0 {
            let selected = self.debug_edit_index.min(count - 1);
            let start = selected / per_page * per_page;
            for (offset, key) in keys.iter().skip(start).take(per_page).enumerate() {
                let y = top + 78.0 + offset as f32 * line_h;
                if start + offset == selected {
                    draw_rectangle(left + pad / 2.0, y - line_h * 0.4, box_w - pad, line_h,
                        Color::new(0.16, 0.20, 0.32, 1.0));
                }
                let value = config::value(key).unwrap_or_default();
                draw_text_at(&format!("{key}: {value}"), left + pad, y, size,
                    if start + offset == selected { INV_ACCENT_COLOR } else { TEXT_COLOR },
                    &self.assets.font, 0.0, 0.5);
            }
        }

        let footer = if let Some(buffer) = &self.debug_edit_value {
            let key = keys.get(self.debug_edit_index.min(count.saturating_sub(1)))
                .cloned().unwrap_or_default();
            format!("{key}: {buffer}_   ENTER save   ESC cancel")
        } else if !self.debug_message.is_empty() {
            self.debug_message.clone()
        } else {
            String::new()
        };
        if !footer.is_empty() {
            draw_text_at(&footer, left + pad, top + box_h - 22.0, 12, INV_ACCENT_COLOR,
                &self.assets.font, 0.0, 0.5);
        }
    }

    fn draw_help(&self) {
        let rows = [
            "CONTROLS",
            "WASD move   E/Enter interact   Z attack",
            "I inventory   Tab switch inventory/equipment",
            "Arrow keys navigate menus and inventory",
            "H spawn a nearby enemy",
            "F1 toggle this help   Esc/Enter close",
            "M minimize debug   E edit config while expanded",
            "R reset selected config   A reset all config",
            "1-6 choose save slot   S Settings in menu",
        ];
        let box_w = (screen_width() - 80.0).min(620.0);
        let box_h = rows.len() as f32 * 22.0 + 46.0;
        let left = (screen_width() - box_w) / 2.0;
        let top = (screen_height() - box_h) / 2.0;
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), INV_DIM_COLOR);
        draw_rectangle(left, top, box_w, box_h, PANEL_COLOR);
        draw_rectangle_lines(left, top, box_w, box_h, 2.0, INV_ACCENT_COLOR);
        for (index, row) in rows.iter().enumerate() {
            let color = if index == 0 { INV_ACCENT_COLOR } else { TEXT_COLOR };
            let size = if index == 0 { 20 } else { 12 };
            draw_text_at(row, left + 20.0, top + 34.0 + index as f32 * 22.0,
                size, color, &self.assets.font, 0.0, 0.5);
        }
    }

    fn draw_close_prompt(&self) {
        let box_w = (screen_width() - 80.0).min(520.0);
        let box_h = 170.0;
        let left = (screen_width() - box_w) / 2.0;
        let top = (screen_height() - box_h) / 2.0;
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), INV_DIM_COLOR);
        draw_rectangle(left, top, box_w, box_h, PANEL_COLOR);
        draw_rectangle_lines(left, top, box_w, box_h, 2.0, INV_BORDER_COLOR);
        draw_text_at("UNSAVED CHANGES", screen_width() / 2.0, top + 38.0,
            22, INV_ACCENT_COLOR, &self.assets.font, 0.5, 0.5);
        draw_text_at("Quit without saving?", screen_width() / 2.0, top + 78.0,
            14, TEXT_COLOR, &self.assets.font, 0.5, 0.5);
        draw_text_at("Y / ENTER - quit       N / ESC - stay", screen_width() / 2.0,
            top + box_h - 40.0, 12, MENU_MUTED_TEXT, &self.assets.font, 0.5, 0.5);
    }

    fn draw_new_game_prompt(&self) {
        let box_w = (screen_width() - 80.0).min(520.0);
        let box_h = 170.0;
        let left = (screen_width() - box_w) / 2.0;
        let top = (screen_height() - box_h) / 2.0;
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), INV_DIM_COLOR);
        draw_rectangle(left, top, box_w, box_h, PANEL_COLOR);
        draw_rectangle_lines(left, top, box_w, box_h, 2.0, INV_BORDER_COLOR);
        draw_text_at("NEW GAME", screen_width() / 2.0, top + 38.0,
            22, INV_ACCENT_COLOR, &self.assets.font, 0.5, 0.5);
        draw_text_at("A current game exists; data will be lost.", screen_width() / 2.0, top + 78.0,
            14, TEXT_COLOR, &self.assets.font, 0.5, 0.5);
        draw_text_at("Y / ENTER - start new game       N / ESC - cancel", screen_width() / 2.0,
            top + box_h - 40.0, 12, MENU_MUTED_TEXT, &self.assets.font, 0.5, 0.5);
    }

    fn camera_target(&self) -> Vec2 {
        let half_w = screen_width() / (2.0 * crate::config::get().camera_zoom);
        let half_h = screen_height() / (2.0 * crate::config::get().camera_zoom);
        let world_w = self.map.width as f32 * TILE_SIZE;
        let world_h = self.map.height as f32 * TILE_SIZE;
        let mut target = self.camera + self.freecam;
        target.x = if world_w <= half_w * 2.0 {
            world_w / 2.0
        } else {
            target.x.clamp(half_w, world_w - half_w)
        };
        target.y = if world_h <= half_h * 2.0 {
            world_h / 2.0
        } else {
            target.y.clamp(half_h, world_h - half_h)
        };
        target
    }

    fn set_world_camera(&self) {
        let sw = screen_width();
        let sh = screen_height();
        set_camera(&Camera2D {
            target: self.camera_target(),
            zoom: vec2(2.0 * crate::config::get().camera_zoom / sw, 2.0 * crate::config::get().camera_zoom / sh),
            ..Default::default()
        });
    }

    fn draw_game(&self) {
        self.set_world_camera();
        self.map.draw(&self.assets.tiles);
        let visible = if self.room_mode() && !self.cli.disable_fog_of_war {
            Some(&self.visible_tiles)
        } else {
            None
        };
        self.enemies.draw(&self.assets, visible, &self.map);
        self.player.draw(&self.assets);
        self.draw_attack_effects();
        self.draw_fog();
        self.draw_damage_numbers();
        set_default_camera();
        self.draw_hud();
        if self.inventory_open {
            self.draw_inventory();
        }
        self.draw_dialogue();
        self.draw_options();
        if let Some(transition) = self.transition {
            let duration = crate::config::get().transition_duration.max(f32::EPSILON);
            let (elapsed, fading_out) = match transition {
                MapTransition::FadeOut { elapsed, .. } => (elapsed, true),
                MapTransition::FadeIn { elapsed } => (elapsed, false),
            };
            let progress = (elapsed / duration).clamp(0.0, 1.0);
            let alpha = if fading_out { progress } else { 1.0 - progress };
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, alpha));
        }
    }

    fn draw_attack_effects(&self) {
        for effect in &self.attack_effects {
            let alpha = (effect.timer / crate::config::get().attack_effect_duration).clamp(0.0, 1.0);
            draw_rectangle(
                effect.pos.x - crate::config::get().attack_effect_size / 2.0,
                effect.pos.y - crate::config::get().attack_effect_size / 2.0,
                crate::config::get().attack_effect_size,
                crate::config::get().attack_effect_size,
                Color::new(1.0, 220.0 / 255.0, 120.0 / 255.0, alpha),
            );
        }
    }

    fn draw_damage_numbers(&self) {
        for number in &self.damage_numbers {
            let progress = (number.timer / crate::config::get().damage_number_lifetime).clamp(0.0, 1.0);
            let alpha = (progress * 2.0).min(1.0);
            let shade = 60.0 + 80.0 * (1.0 - progress);
            draw_text_at(
                &number.text,
                number.pos.x,
                number.pos.y,
                crate::config::get().damage_number_font_size,
                Color::new(1.0, shade / 255.0, shade / 255.0, alpha),
                &self.assets.font,
                0.5,
                0.5,
            );
        }
    }

    fn cast_ray(
        &self,
        x0: f32,
        y0: f32,
        angle: f32,
        max_distance: f32,
        solid: &HashSet<Tile>,
        lightable: &HashSet<Tile>,
    ) -> f32 {
        if solid.is_empty() {
            return max_distance;
        }
        let tw = self.map.tile_width;
        let th = self.map.tile_height;
        let dx = angle.cos();
        let dy = angle.sin();
        let step_x = if dx > 0.0 { 1 } else { -1 };
        let step_y = if dy > 0.0 { 1 } else { -1 };
        let mut col = (x0 / tw).floor() as i32;
        let mut row = (y0 / th).floor() as i32;

        let (mut t_max_x, t_delta_x) = if dx != 0.0 {
            let boundary = (col + if dx > 0.0 { 1 } else { 0 }) as f32 * tw;
            ((boundary - x0) / dx, tw / dx.abs())
        } else {
            (f32::INFINITY, f32::INFINITY)
        };
        let (mut t_max_y, t_delta_y) = if dy != 0.0 {
            let boundary = (row + if dy > 0.0 { 1 } else { 0 }) as f32 * th;
            ((boundary - y0) / dy, th / dy.abs())
        } else {
            (f32::INFINITY, f32::INFINITY)
        };

        let mut distance = 0.0f32;
        let mut lightable_hit = false;
        loop {
            if (t_max_x - t_max_y).abs() < 1e-6 {
                let next_col = col + step_x;
                let next_row = row + step_y;
                if solid.contains(&Tile(next_col, row))
                    || solid.contains(&Tile(col, next_row))
                {
                    return distance.min(max_distance);
                }
                col = next_col;
                row = next_row;
                t_max_x += t_delta_x;
                t_max_y += t_delta_y;
            } else if t_max_x < t_max_y {
                col += step_x;
                distance = t_max_x;
                t_max_x += t_delta_x;
            } else {
                row += step_y;
                distance = t_max_y;
                t_max_y += t_delta_y;
            }
            if distance > max_distance {
                return max_distance;
            }
            let tile = Tile(col, row);
            if col < 0 || row < 0 || col >= self.map.width || row >= self.map.height {
                return distance;
            }
            if solid.contains(&tile) {
                if lightable.contains(&tile) {
                    lightable_hit = true;
                } else {
                    return distance;
                }
            } else if lightable_hit {
                return distance;
            }
        }
    }

    fn light_emit_positions(&self) -> Vec<Vec2> {
        let tw = self.map.tile_width;
        let th = self.map.tile_height;
        let radius = crate::config::get().light_radius_tiles;
        let solid = &self.map.light_blockers;
        let open = |col: i32, row: i32| {
            col >= 0
                && row >= 0
                && col < self.map.width
                && row < self.map.height
                && !solid.contains(&Tile(col, row))
        };
        let mut result = Vec::new();
        for source in &self.map.light_sources {
            let col = (source.x / tw).floor() as i32;
            let row = (source.y / th).floor() as i32;
            if open(col, row) {
                result.push(*source);
                continue;
            }
            let left = open(col - 1, row);
            let right = open(col + 1, row);
            let up = open(col, row - 1);
            let down = open(col, row + 1);
            let candidates = if !left && !right && up && down {
                vec![(0, -1), (0, 1)]
            } else if !up && !down && left && right {
                vec![(-1, 0), (1, 0)]
            } else {
                result.push(*source);
                continue;
            };
            let mut best = (col, row);
            let mut best_area = -1i32;
            for (offset_col, offset_row) in candidates {
                let near_col = col + offset_col;
                let near_row = row + offset_row;
                let mut area = 0;
                for area_col in near_col - radius as i32..=near_col + radius as i32 {
                    for area_row in near_row - radius as i32..=near_row + radius as i32 {
                        if open(area_col, area_row) {
                            area += 1;
                        }
                    }
                }
                if area > best_area {
                    best_area = area;
                    best = (near_col, near_row);
                }
            }
            result.push(vec2(
                best.0 as f32 * tw + tw / 2.0,
                best.1 as f32 * th + th / 2.0,
            ));
        }
        result
    }

    fn ray_squares(
        &self,
        x: f32,
        y: f32,
        angles: &[f32],
        max_distance: f32,
        solid: &HashSet<Tile>,
        lightable: &HashSet<Tile>,
    ) -> Vec<f32> {
        angles
            .iter()
            .map(|angle| {
                let distance = self.cast_ray(x, y, *angle, max_distance, solid, lightable);
                distance * distance
            })
            .collect()
    }

    fn radial_light_at(dx: f32, dy: f32, rays: &[f32], radius: f32, peak: f32) -> f32 {
        let distance_squared = dx * dx + dy * dy;
        let radius_squared = radius * radius;
        if rays.is_empty() || distance_squared >= radius_squared {
            return 0.0;
        }
        let count = rays.len();
        let angle = dy.atan2(dx).rem_euclid(std::f32::consts::TAU);
        let index = (angle * (count as f32 / std::f32::consts::TAU)) as usize % count;
        if distance_squared > rays[index] {
            return 0.0;
        }
        (1.0 - distance_squared / radius_squared) * peak
    }

    fn cone_light_at(
        dx: f32,
        dy: f32,
        facing_angle: f32,
        rays: &[f32],
        radius: f32,
        peak: f32,
        half_angle: f32,
        edge_fraction: f32,
        inner_radius: f32,
    ) -> f32 {
        let distance_squared = dx * dx + dy * dy;
        let radius_squared = radius * radius;
        let mut value = 0.0;
        if !rays.is_empty() && distance_squared < radius_squared {
            let angle = dy.atan2(dx);
            let offset = (angle - facing_angle + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            if offset.abs() <= half_angle {
                let count = rays.len();
                let index = if count > 1 {
                    let step = 2.0 * half_angle / (count - 1) as f32;
                    (((offset + half_angle) / step).round() as isize)
                        .clamp(0, count as isize - 1) as usize
                } else {
                    0
                };
                if distance_squared <= rays[index] {
                    let mut falloff = 1.0 - distance_squared / radius_squared;
                    let edge = half_angle * edge_fraction;
                    if edge > 0.0 {
                        falloff *= ((half_angle - offset.abs()) / edge).clamp(0.0, 1.0);
                    }
                    value = falloff.max(0.0) * peak;
                }
            }
        }
        let inner = (1.0 - distance_squared / (inner_radius * inner_radius)).clamp(0.0, 1.0) * peak;
        value.max(inner)
    }

    fn draw_nonroom_darkness(&self) {
        let cfg = config::get();
        let tw = self.map.tile_width;
        let th = self.map.tile_height;
        let step = tw / cfg.lightmap_samples_per_tile;
        let target = self.camera_target();
        let half_w = screen_width() / (2.0 * cfg.camera_zoom);
        let half_h = screen_height() / (2.0 * cfg.camera_zoom);
        let left = ((target.x - half_w - tw) / step).floor() * step;
        let right = ((target.x + half_w + tw) / step).ceil() * step;
        let top = ((target.y - half_h - th) / step).floor() * step;
        let bottom = ((target.y + half_h + th) / step).ceil() * step;

        let lights = if self.cli.disable_map_light {
            Vec::new()
        } else {
            self.light_emit_positions()
        };
        let light_radius = cfg.light_radius_tiles * tw;
        let visible_lights: Vec<usize> = lights
            .iter()
            .enumerate()
            .filter(|(_, light)| {
                light.x + light_radius >= left
                    && light.x - light_radius <= right
                    && light.y + light_radius >= top
                    && light.y - light_radius <= bottom
            })
            .map(|(index, _)| index)
            .collect();

        let player_origin = vec2(
            (self.player.pos.x / (tw / 4.0)).round() * (tw / 4.0),
            (self.player.pos.y / (th / 4.0)).round() * (th / 4.0),
        );
        let player_rays = if self.cli.disable_player_light {
            Vec::new()
        } else {
            let offset = self.player.facing_offset();
            let facing_angle = offset.y.atan2(offset.x);
            let half = cfg.player_light_cone_degrees.to_radians() / 2.0;
            let count = cfg.player_ray_count.max(1);
            let angles: Vec<f32> = (0..count)
                .map(|index| {
                    if count > 1 {
                        facing_angle - half + 2.0 * half * index as f32 / (count - 1) as f32
                    } else {
                        facing_angle
                    }
                })
                .collect();
            let blockers = self.map.player_light_blockers();
            let lightable = self.map.lightable_for_lights();
            self.ray_squares(
                player_origin.x,
                player_origin.y,
                &angles,
                cfg.player_vision_tiles * tw,
                &blockers,
                &lightable,
            )
        };
        let player_offset = self.player.facing_offset();
        let player_facing = player_offset.y.atan2(player_offset.x);

        let light_rays: Vec<Vec<f32>> = visible_lights
            .iter()
            .map(|index| {
                let light = lights[*index];
                let count = cfg.light_ray_count.max(1);
                let angles: Vec<f32> = (0..count)
                    .map(|ray| std::f32::consts::TAU * ray as f32 / count as f32)
                    .collect();
                let lightable = self.map.lightable_for_lights();
                self.ray_squares(
                    light.x,
                    light.y,
                    &angles,
                    light_radius,
                    &self.map.light_blockers,
                    &lightable,
                )
            })
            .collect();

        let half_angle = cfg.player_light_cone_degrees.to_radians() / 2.0;
        let inner_radius = cfg.player_light_inner_tiles * tw;
        let mut y = top + step / 2.0;
        while y <= bottom {
            let mut x = left + step / 2.0;
            while x <= right {
                let mut light = 0.0f32;
                let mut glow = 0.0f32;
                if !player_rays.is_empty() {
                    light += Self::cone_light_at(
                        x - player_origin.x,
                        y - player_origin.y,
                        player_facing,
                        &player_rays,
                        cfg.player_vision_tiles * tw,
                        cfg.nonroom_player_light_peak,
                        half_angle,
                        cfg.player_light_edge_fraction,
                        inner_radius,
                    );
                }
                for (index, rays) in light_rays.iter().enumerate() {
                    let light_pos = lights[visible_lights[index]];
                    let dx = x - light_pos.x;
                    let dy = y - light_pos.y;
                    light += Self::radial_light_at(dx, dy, rays, light_radius, cfg.light_peak);
                    let glow_radius = cfg.light_source_tint_radius_tiles * tw;
                    let distance = (dx * dx + dy * dy).sqrt();
                    if distance < glow_radius {
                        let kernel = 1.0 - distance / glow_radius;
                        glow += kernel * kernel;
                    }
                }
                light = light.clamp(0.0, 1.0);
                glow = (glow * light * 1.5).clamp(0.0, 1.0);
                let alpha = cfg.darkness_alpha as f32 * (1.0 - light) * (1.0 - glow)
                    + cfg.light_source_tint_alpha as f32 * glow;
                if alpha > 1.0 {
                    let tint = LIGHT_SOURCE_TINT_COLOR;
                    draw_rectangle(
                        x - step / 2.0,
                        y - step / 2.0,
                        step,
                        step,
                        Color::new(tint.r * glow, tint.g * glow, tint.b * glow, alpha / 255.0),
                    );
                }
                x += step;
            }
            y += step;
        }
    }

    fn draw_fog(&self) {
        if !self.room_mode() {
            self.draw_nonroom_darkness();
            return;
        }
        let target = self.camera_target();
        let half_w = screen_width() / (2.0 * crate::config::get().camera_zoom);
        let half_h = screen_height() / (2.0 * crate::config::get().camera_zoom);
        let step = TILE_SIZE / 4.0;
        let left = ((target.x - half_w - TILE_SIZE) / step).floor() * step;
        let right = ((target.x + half_w + TILE_SIZE) / step).ceil() * step;
        let top = ((target.y - half_h - TILE_SIZE) / step).floor() * step;
        let bottom = ((target.y + half_h + TILE_SIZE) / step).ceil() * step;
        let radius = crate::config::get().player_light_radius_tiles * TILE_SIZE;
        let radius_sq = radius * radius;
        let cols = self.map.width;
        let rows = self.map.height;
        let mut y = top + step / 2.0;
        while y < bottom {
            let row = (y / TILE_SIZE).floor() as i32;
            let mut x = left + step / 2.0;
            while x < right {
                let col = (x / TILE_SIZE).floor() as i32;
                let mut level = 0.0;
                if col >= 0 && row >= 0 && col < cols && row < rows {
                    level = self.room_light[(row * cols + col) as usize];
                }
                if !self.cli.disable_player_light {
                    let dx = x - self.player.pos.x;
                    let dy = y - self.player.pos.y;
                    let distance_sq = dx * dx + dy * dy;
                    if distance_sq < radius_sq {
                        level += crate::config::get().player_light_peak * (1.0 - distance_sq.sqrt() / radius);
                    }
                }
                let alpha = (crate::config::get().darkness_alpha as f32 * (1.0 - level.min(1.0))) as u8;
                if alpha > 2 {
                    draw_rectangle(
                        x - step / 2.0,
                        y - step / 2.0,
                        step,
                        step,
                        Color::new(0.0, 0.0, 0.0, alpha as f32 / 255.0),
                    );
                }
                x += step;
            }
            y += step;
        }
    }

    fn draw_hud(&self) {
        let heart_count = self.player.max_hp / crate::config::get().health_per_heart;
        let moving_input = get_keys_down().iter().any(|key| {
            matches!(key, KeyCode::W | KeyCode::A | KeyCode::S | KeyCode::D)
        });
        let alpha = if moving_input { 100.0 / 255.0 } else { 1.0 };
        let hearts_left = (self.player.hp + crate::config::get().health_per_heart - 1)
            / crate::config::get().health_per_heart;
        let shaking = self.player.hp > 0 && hearts_left <= 3;
        for index in 0..heart_count {
            let texture = if self.player.hp > index * crate::config::get().health_per_heart {
                &self.assets.heart
            } else {
                &self.assets.broken_heart
            };
            let mut center_x = 20.0 + index as f32 * (30.0 + 2.0);
            let mut center_y = 20.0;
            if shaking {
                center_x += (self.heart_shake_time * 50.0 + index as f32).sin() * 2.0;
                center_y += (self.heart_shake_time * 60.0 + index as f32).cos() * 2.0;
            }
            draw_texture_ex(
                texture,
                center_x - 15.0,
                center_y - 15.0,
                Color::new(1.0, 1.0, 1.0, alpha),
                DrawTextureParams {
                    dest_size: Some(vec2(30.0, 30.0)),
                    ..Default::default()
                },
            );
        }
        let (attack, defense) = self.equipment.combat_stats(&self.items);
        for (label, value, left) in [("ATK", attack, 20.0), ("DEF", defense, 78.0)] {
            draw_rectangle(left, 34.0, 52.0, 24.0, Color::new(72.0 / 255.0, 76.0 / 255.0, 88.0 / 255.0, 1.0));
            draw_text_at(label, left + 6.0, 46.0, 10, TEXT_COLOR, &self.assets.font, 0.0, 0.5);
            draw_text_at(&value.to_string(), left + 34.0, 46.0, 12, TEXT_COLOR, &self.assets.font, 0.5, 0.5);
        }
    }

    fn draw_dialogue(&self) {
        if !self.dialogue.open && !self.options.open {
            return;
        }
        let scale = 2.0;
        let w = self.assets.dialogue_box.width() * scale;
        let h = self.assets.dialogue_box.height() * scale;
        let x = (screen_width() - w) / 2.0;
        let y = screen_height() - h - 40.0;
        draw_texture_ex(
            &self.assets.dialogue_box,
            x,
            y,
            WHITE,
            DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() },
        );
        draw_wrapped_text(
            &self.dialogue.visible(),
            x + 40.0,
            y + 40.0,
            18,
            TEXT_COLOR,
            &self.assets.font,
            w - 80.0,
            22.0,
        );
    }

    fn draw_options(&self) {
        if !self.options.open || !self.dialogue.complete() {
            return;
        }
        let w = self.assets.dialogue_box.width() * 2.0;
        let h = self.assets.dialogue_box.height() * 2.0;
        let x = (screen_width() - w) / 2.0;
        let y = screen_height() - h - 40.0;
        let parts: Vec<String> = self
            .options
            .choices
            .iter()
            .enumerate()
            .map(|(index, choice)| {
                if index == self.options.index {
                    format!("> {choice}")
                } else {
                    choice.clone()
                }
            })
            .collect();
        draw_text_at(
            &parts.join("  "),
            x + 40.0,
            y + h - 40.0,
            18,
            TEXT_COLOR,
            &self.assets.font,
            0.0,
            1.0,
        );
    }

    fn draw_inventory(&self) {
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), INV_DIM_COLOR);

        let equip_grid_h = 3.0 * crate::config::get().equipment_slot_size + 2.0 * crate::config::get().equipment_slot_gap;
        let equip_panel_w = crate::config::get().equipment_slot_size + crate::config::get().panel_padding * 2.0;
        let equip_panel_h = equip_grid_h + crate::config::get().panel_padding * 2.0 + crate::config::get().label_height;
        let inv_grid_w = crate::config::get().columns as f32 * crate::config::get().slot_size + (crate::config::get().columns as f32 - 1.0) * crate::config::get().slot_gap;
        let inv_grid_h = crate::config::get().rows as f32 * crate::config::get().slot_size + (crate::config::get().rows as f32 - 1.0) * crate::config::get().slot_gap;
        let inv_panel_w = inv_grid_w + crate::config::get().panel_padding * 2.0;
        let inv_panel_h = inv_grid_h + crate::config::get().panel_padding * 2.0 + crate::config::get().label_height;
        let total_w = equip_panel_w + crate::config::get().panel_gap + inv_panel_w;
        let total_h = equip_panel_h.max(inv_panel_h);
        let left = (screen_width() - total_w) / 2.0;
        let top = (screen_height() - total_h) / 2.0;
        let bottom = top + total_h;

        let outer_left = left - crate::config::get().panel_gap / 2.0;
        let outer_right = left + total_w + crate::config::get().panel_gap / 2.0;
        let outer_top = top - crate::config::get().panel_gap / 2.0 - 22.0;
        let outer_bottom = bottom + crate::config::get().panel_gap / 2.0;
        draw_rectangle(outer_left, outer_top, outer_right - outer_left, outer_bottom - outer_top, PANEL_COLOR);
        draw_rectangle_lines(outer_left, outer_top, outer_right - outer_left, outer_bottom - outer_top, 2.0, INV_BORDER_COLOR);
        draw_text_at("INVENTORY", screen_width() / 2.0, top - crate::config::get().panel_gap / 2.0 - 11.0, 22, INV_ACCENT_COLOR, &self.assets.font, 0.5, 0.5);

        self.draw_equipment_panel(left, top, equip_panel_w, equip_panel_h, equip_grid_h);
        self.draw_inventory_panel(left + equip_panel_w + crate::config::get().panel_gap, top, inv_panel_w, inv_panel_h, inv_grid_h);
    }

    fn draw_equipment_panel(&self, left: f32, top: f32, width: f32, height: f32, grid_h: f32) {
        let focused = self.inv_focus == InvFocus::Equipment;
        draw_rectangle(left, top, width, height, PANEL_COLOR);
        draw_rectangle_lines(left, top, width, height, 2.0, INV_BORDER_COLOR);
        draw_rectangle(left, top, width, crate::config::get().panel_padding, INV_HEADER_COLOR);
        draw_text_at("EQUIPMENT", left + crate::config::get().panel_padding / 2.0, top + crate::config::get().panel_padding / 2.0, 13, INV_ACCENT_COLOR, &self.assets.font, 0.0, 0.5);
        for (index, slot) in config::EQUIPMENT_SLOTS.iter().enumerate() {
            let x = left + crate::config::get().panel_padding;
            let y = top + crate::config::get().panel_padding + index as f32 * (crate::config::get().equipment_slot_size + crate::config::get().equipment_slot_gap);
            let selected = focused && index == self.equip_index;
            draw_rectangle(x, y, crate::config::get().equipment_slot_size, crate::config::get().equipment_slot_size,
                if selected { SLOT_SELECTED_COLOR } else { EQUIPMENT_SLOT_BG });
            draw_rectangle_lines(x, y, crate::config::get().equipment_slot_size, crate::config::get().equipment_slot_size,
                if selected { 2.0 } else { 1.0 },
                if selected { INV_SELECTED_BORDER } else { INV_SLOT_BORDER });
            let occupied = self.equipment.get(slot).is_some();
            let slot_tint = if occupied {
                Color::new(0.65, 0.68, 0.78, 1.0)
            } else {
                WHITE
            };
            if let Some(texture) = self.assets.slot_textures.get(*slot) {
                draw_texture_ex(texture, x, y, slot_tint,
                    DrawTextureParams { dest_size: Some(vec2(crate::config::get().equipment_slot_size, crate::config::get().equipment_slot_size)), ..Default::default() });
            }
            if let Some(item_id) = self.equipment.get(slot) {
                if let Some(item) = self.items.get(item_id) {
                    if let Some(texture) = self.assets.item_texture(&item.texture) {
                        let size = 32.0;
                        draw_texture_ex(&texture, x + (crate::config::get().equipment_slot_size - size) / 2.0,
                            y + (crate::config::get().equipment_slot_size - size) / 2.0, WHITE,
                            DrawTextureParams { dest_size: Some(vec2(size, size)), ..Default::default() });
                    }
                }
            }
        }
        if focused {
            let slot = config::EQUIPMENT_SLOTS[self.equip_index];
            if let Some(item_id) = self.equipment.get(slot) {
                if let Some(item) = self.items.get(item_id) {
                    let x = left + crate::config::get().panel_padding;
                    let y = top + crate::config::get().panel_padding + self.equip_index as f32 * (crate::config::get().equipment_slot_size + crate::config::get().equipment_slot_gap);
                    draw_quality(
                        x + (crate::config::get().equipment_slot_size - (3.0 * crate::config::get().star_size + 2.0 * crate::config::get().star_gap)) / 2.0,
                        y + crate::config::get().equipment_slot_size + crate::config::get().star_size + 1.0,
                        item.quality,
                    );
                }
            }
        }
        let slot = config::EQUIPMENT_SLOTS[self.equip_index];
        let mut label = match self.equipment.get(slot) {
            Some(id) => self.items.get(id).map(|i| i.name.clone()).unwrap_or_else(|| slot.to_string()),
            None => {
                let mut title = slot.to_string();
                if let Some(first) = title.get_mut(0..1) {
                    first.make_ascii_uppercase();
                }
                title
            }
        };
        if self.equipment.get(slot).is_some() {
            label.push_str(" · Enter");
        }
        draw_text_at(&label, left + crate::config::get().panel_padding, top + height - crate::config::get().label_height / 2.0,
            12, TEXT_COLOR, &self.assets.font, 0.0, 0.5);
        let _ = grid_h;
    }

    fn draw_inventory_panel(&self, left: f32, top: f32, width: f32, height: f32, grid_h: f32) {
        let focused = self.inv_focus == InvFocus::Inventory;
        draw_rectangle(left, top, width, height, PANEL_COLOR);
        draw_rectangle_lines(left, top, width, height, 2.0, INV_BORDER_COLOR);
        draw_rectangle(left, top, width, crate::config::get().panel_padding, INV_HEADER_COLOR);
        draw_text_at("ITEMS", left + crate::config::get().panel_padding / 2.0, top + crate::config::get().panel_padding / 2.0, 13, INV_ACCENT_COLOR, &self.assets.font, 0.0, 0.5);
        draw_text_at(&format!("Gold: {}", self.inventory.gold), left + width - crate::config::get().panel_padding,
            top + crate::config::get().panel_padding / 2.0, 12, INV_ACCENT_COLOR, &self.assets.font, 1.0, 0.5);

        for index in 0..self.inventory.slots.len() {
            let col = (index % crate::config::get().columns) as f32;
            let row = (index / crate::config::get().columns) as f32;
            let x = left + crate::config::get().panel_padding + col * (crate::config::get().slot_size + crate::config::get().slot_gap);
            let y = top + crate::config::get().panel_padding + row * (crate::config::get().slot_size + crate::config::get().slot_gap);
            let selected = focused && index == self.inv_index;
            draw_rectangle(x, y, crate::config::get().slot_size, crate::config::get().slot_size,
                if selected { SLOT_SELECTED_COLOR } else { SLOT_COLOR });
            draw_rectangle_lines(x, y, crate::config::get().slot_size, crate::config::get().slot_size,
                if selected { 2.0 } else { 1.0 },
                if selected { INV_SELECTED_BORDER } else { INV_SLOT_BORDER });
            if let Some(stack) = self.inventory.slots.get(index).and_then(|s| s.as_ref()) {
                if let Some(item) = self.items.get(&stack.item_id) {
                    if let Some(texture) = self.assets.item_texture(&item.texture) {
                        let size = 32.0;
                        draw_texture_ex(&texture, x + (crate::config::get().slot_size - size) / 2.0, y + (crate::config::get().slot_size - size) / 2.0, WHITE,
                            DrawTextureParams { dest_size: Some(vec2(size, size)), ..Default::default() });
                    }
                    if stack.count > 1 {
                        draw_text_at(&stack.count.to_string(), x + crate::config::get().slot_size - 4.0, y + crate::config::get().slot_size - 4.0,
                            12, TEXT_COLOR, &self.assets.font, 1.0, 1.0);
                    }
                }
            }
        }

        if focused {
            if let Some(stack) = self.inventory.slots.get(self.inv_index).and_then(|s| s.as_ref()) {
                if let Some(item) = self.items.get(&stack.item_id) {
                    let col = (self.inv_index % crate::config::get().columns) as f32;
                    let row = (self.inv_index / crate::config::get().columns) as f32;
                    let x = left + crate::config::get().panel_padding + col * (crate::config::get().slot_size + crate::config::get().slot_gap);
                    let y = top + crate::config::get().panel_padding + row * (crate::config::get().slot_size + crate::config::get().slot_gap);
                    draw_quality(
                        x + (crate::config::get().slot_size - (3.0 * crate::config::get().star_size + 2.0 * crate::config::get().star_gap)) / 2.0,
                        y + crate::config::get().slot_size + crate::config::get().star_size + 1.0,
                        item.quality,
                    );
                }
            }
        }

        let mut label = "—".to_string();
        if let Some(stack) = self.inventory.slots.get(self.inv_index).and_then(|s| s.as_ref()) {
            if let Some(item) = self.items.get(&stack.item_id) {
                label = item.name.clone();
                if stack.count > 1 {
                    label.push_str(&format!(" ×{}", stack.count));
                }
                let mut hint = String::new();
                if item.item_type == "consumable" {
                    hint = "Enter use".into();
                } else if item.slot.is_some() {
                    hint = "Enter equip".into();
                }
                if item.removeable {
                    if !hint.is_empty() {
                        hint.push_str(" · ");
                    }
                    hint.push_str(&format!("R sell {}g", item.price));
                } else if item.item_type == "key" {
                    hint = "Cannot sell".into();
                }
                if !hint.is_empty() {
                    label.push_str(&format!(" · {hint}"));
                }
            }
        }
        draw_text_at(&label, left + crate::config::get().panel_padding, top + height - crate::config::get().label_height / 2.0,
            12, TEXT_COLOR, &self.assets.font, 0.0, 0.5);
        let _ = grid_h;
    }

    fn menu_primary_action(&mut self) {
        if self.can_resume {
            self.screen = Screen::Game;
            self.new_game_prompt = false;
        } else if self.active_run {
            self.new_game_prompt = true;
        } else {
            self.start_game();
        }
    }

    fn settings_rows(&self) -> Vec<(String, String)> {
        let on_off = |enabled: bool| if enabled { "on" } else { "off" }.to_string();
        vec![
            ("Start map".into(), format!("map{}", self.cli.level + 1)),
            ("Startup script".into(), self.cli.script.clone().unwrap_or_else(|| "none".into())),
            ("Infinite health".into(), on_off(self.cli.health_infinite)),
            ("Freecam".into(), on_off(self.cli.freecam)),
            ("Player light".into(), on_off(!self.cli.disable_player_light)),
            ("Map lights".into(), on_off(!self.cli.disable_map_light)),
            ("Room visibility".into(), on_off(!self.cli.disable_rooms)),
            ("Fog of war".into(), on_off(!self.cli.disable_fog_of_war)),
            ("Enemies".into(), on_off(!self.cli.no_enemies)),
            ("Debug overlay".into(), on_off(self.cli.debug)),
            ("Output log file".into(), on_off(self.cli.output_file)),
            ("Back".into(), String::new()),
        ]
    }

    fn startup_script_index(&self) -> usize {
        match &self.cli.script {
            None => 0,
            Some(name) => self
                .available_scripts
                .iter()
                .position(|script| script == name)
                .map(|index| index + 1)
                .unwrap_or(0),
        }
    }

    fn change_setting(&mut self, index: usize, delta: i32) {
        if index == SETTING_BACK || index >= self.settings_rows().len() {
            return;
        }
        let label = self.settings_rows()[index].0.clone();
        match index {
            SETTING_START_MAP => {
                let count = config::MAP_FILES.len() as i32;
                let next = (self.cli.level as i32 + delta).rem_euclid(count) as usize;
                self.cli.level = next;
                self.cli.level_given = true;
                self.settings_message = "Applies to the next new game.".into();
            }
            SETTING_START_SCRIPT => {
                let count = self.available_scripts.len() as i32 + 1;
                let next = (self.startup_script_index() as i32 + delta).rem_euclid(count) as usize;
                self.cli.script = if next == 0 {
                    None
                } else {
                    Some(self.available_scripts[next - 1].clone())
                };
                self.settings_message = "Applies to the next new game.".into();
            }
            SETTING_HEALTH => {
                self.cli.health_infinite = !self.cli.health_infinite;
                self.player.invincible = self.cli.health_infinite;
            }
            SETTING_FREECAM => {
                self.cli.freecam = !self.cli.freecam;
                self.freecam = Vec2::ZERO;
                if !self.cli.freecam {
                    self.camera = self.player.pos;
                }
            }
            SETTING_PLAYER_LIGHT => self.cli.disable_player_light = !self.cli.disable_player_light,
            SETTING_MAP_LIGHT => self.cli.disable_map_light = !self.cli.disable_map_light,
            SETTING_ROOMS => {
                self.cli.disable_rooms = !self.cli.disable_rooms;
                self.room_light_ready = false;
                self.visible_tiles.clear();
                self.visible_rooms.clear();
            }
            SETTING_FOG => self.cli.disable_fog_of_war = !self.cli.disable_fog_of_war,
            SETTING_NO_ENEMIES => {
                self.cli.no_enemies = !self.cli.no_enemies;
                self.enemies.set_disabled(self.cli.no_enemies);
            }
            SETTING_DEBUG => self.cli.debug = !self.cli.debug,
            SETTING_OUTPUT => {
                self.cli.output_file = !self.cli.output_file;
                crate::set_log_output(self.cli.output_file);
            }
            _ => return,
        }
        if index != SETTING_DEBUG {
            self.cli.debug = true;
        }
        log_line(&format!("[settings] {label}: {}", self.settings_rows()[index].1));
    }

    fn draw_settings(&mut self) {
        let pressed: Vec<KeyCode> = get_keys_pressed().into_iter().collect();
        let rows = self.settings_rows();
        let count = rows.len();
        self.settings_index = self.settings_index.min(count - 1);

        if pressed.contains(&KeyCode::Escape) {
            self.screen = Screen::Menu;
            self.settings_message.clear();
            return;
        }
        if pressed.contains(&KeyCode::Up) || pressed.contains(&KeyCode::W) {
            self.settings_index = (self.settings_index + count - 1) % count;
        }
        if pressed.contains(&KeyCode::Down) || pressed.contains(&KeyCode::S) {
            self.settings_index = (self.settings_index + 1) % count;
        }
        if pressed.contains(&KeyCode::Left) || pressed.contains(&KeyCode::A) {
            self.change_setting(self.settings_index, -1);
        }
        if pressed.contains(&KeyCode::Right) || pressed.contains(&KeyCode::D) {
            self.change_setting(self.settings_index, 1);
        }
        if pressed.contains(&KeyCode::Enter) || pressed.contains(&KeyCode::Space) {
            if self.settings_index == SETTING_BACK {
                self.screen = Screen::Menu;
                self.settings_message.clear();
                return;
            }
            self.change_setting(self.settings_index, 1);
        }

        let (mx, my) = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left) {
            let left = screen_width() / 2.0 - 320.0;
            let top = 150.0;
            let row_height = 42.0;
            let row = ((my - top + row_height / 2.0) / row_height).floor() as isize;
            if row >= 0
                && (row as usize) < count
                && mx >= left - 12.0
                && mx <= left + 648.0
            {
                self.settings_index = row as usize;
                if row as usize == SETTING_BACK {
                    self.screen = Screen::Menu;
                    self.settings_message.clear();
                    return;
                }
                self.change_setting(row as usize, 1);
            }
        }
        if self.screen != Screen::Settings {
            return;
        }

        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.82));
        draw_text_at("SETTINGS", screen_width() / 2.0, 74.0, 30, TEXT_COLOR, &self.assets.font, 0.5, 0.5);
        let left = screen_width() / 2.0 - 320.0;
        let rows = self.settings_rows();
        for (index, (label, value)) in rows.iter().enumerate() {
            let y = 150.0 + index as f32 * 42.0;
            if index == self.settings_index {
                draw_rectangle(left - 12.0, y - 19.0, 648.0, 38.0, Color::new(0.16, 0.18, 0.26, 0.95));
            }
            draw_text_at(label, left, y, 15, TEXT_COLOR, &self.assets.font, 0.0, 0.5);
            if !value.is_empty() {
                let color = if index == self.settings_index { INV_ACCENT_COLOR } else { MENU_MUTED_TEXT };
                draw_text_at(value, left + 620.0, y, 15, color, &self.assets.font, 1.0, 0.5);
            }
        }
        draw_text_at(
            "Up/Down select   Left/Right or Enter change   Esc back",
            screen_width() / 2.0,
            screen_height() - 74.0,
            11,
            MENU_MUTED_TEXT,
            &self.assets.font,
            0.5,
            0.5,
        );
        if !self.settings_message.is_empty() {
            draw_text_at(
                &self.settings_message,
                screen_width() / 2.0,
                screen_height() - 48.0,
                11,
                INV_ACCENT_COLOR,
                &self.assets.font,
                0.5,
                0.5,
            );
        }
    }

    fn draw_menu(&mut self) {
        let title_w = self.assets.title.width() * crate::config::get().title_scale;
        let title_h = self.assets.title.height() * crate::config::get().title_scale;
        draw_texture_ex(&self.assets.title, screen_width() - 40.0 - title_w, 36.0, WHITE,
            DrawTextureParams { dest_size: Some(vec2(title_w, title_h)), ..Default::default() });
        let buttons = [("start", "start"), ("load", "load"), ("save", "save")];
        let (mx, my) = mouse_position();
        let mut actions = Vec::new();
        for (index, (key, label)) in buttons.iter().enumerate() {
            let center_x = 120.0 + 64.0 / 2.0;
            let center_y = 40.0 + 300.0 + 16.0 + index as f32 * 64.0;
            let left = center_x - 64.0;
            let top = center_y - 32.0;
            let hovered = mx >= left && mx <= left + 128.0 && my >= top && my <= top + 64.0;
            let name = if hovered { format!("{key}_hover") } else { key.to_string() };
            if let Some(texture) = self.assets.buttons.get(&name).or_else(|| self.assets.buttons.get(*key)) {
                draw_texture_ex(texture, left, top, WHITE,
                    DrawTextureParams { dest_size: Some(vec2(128.0, 64.0)), ..Default::default() });
            }
            let _ = label;
            if is_mouse_button_pressed(MouseButton::Left) && hovered {
                actions.push((*key).to_string());
            }
        }
        let settings_hovered = mx >= 20.0 && mx <= 84.0
            && my >= screen_height() - 84.0 && my <= screen_height() - 20.0;
        if settings_hovered && is_mouse_button_pressed(MouseButton::Left) {
            self.screen = Screen::Settings;
            self.settings_index = 0;
            self.settings_message.clear();
        }
        let settings_name = if settings_hovered { "settings_hover" } else { "settings" };
        if let Some(texture) = self.assets.buttons.get(settings_name).or_else(|| self.assets.buttons.get("settings")) {
            draw_texture_ex(texture, 20.0, screen_height() - 84.0, WHITE,
                DrawTextureParams { dest_size: Some(vec2(64.0, 64.0)), ..Default::default() });
        }
        let hint_x = screen_width() - 40.0 - title_w / 2.0;
        let start_hint = if self.can_resume {
            "ENTER or SPACE to continue"
        } else {
            "ENTER or SPACE to start"
        };
        draw_text_at(start_hint, hint_x, screen_height() - 110.0,
            10, MENU_MUTED_TEXT, &self.assets.font, 0.5, 0.5);
        draw_text_at("WASD MOVE   E INTERACT   Z ATTACK   H SPAWN ENEMY", hint_x, screen_height() - 74.0,
            10, MENU_MUTED_TEXT, &self.assets.font, 0.5, 0.5);
        draw_text_at("I INVENTORY   TAB EQUIPMENT   ESC MENU   Z SKIP TEXT", hint_x, screen_height() - 56.0,
            10, MENU_MUTED_TEXT, &self.assets.font, 0.5, 0.5);
        draw_text_at(&format!("v{}", crate::config::get().version), screen_width() - 20.0, screen_height() - 20.0,
            10, MENU_MUTED_TEXT, &self.assets.font, 1.0, 1.0);
        if !self.slot_menu_message.is_empty() {
            draw_text_at(&self.slot_menu_message, 120.0, screen_height() - 160.0, 10,
                MENU_MUTED_TEXT, &self.assets.font, 0.0, 0.5);
        }
        let pressed: Vec<KeyCode> = get_keys_pressed().into_iter().collect();
        if pressed.contains(&KeyCode::Enter) || pressed.contains(&KeyCode::Space) {
            self.menu_primary_action();
        }
        if pressed.contains(&KeyCode::Left) {
            self.slot = if self.slot == 1 { crate::config::get().save_slots } else { self.slot - 1 };
        }
        if pressed.contains(&KeyCode::Right) {
            self.slot = self.slot % crate::config::get().save_slots + 1;
        }
        if pressed.contains(&KeyCode::S) {
            self.screen = Screen::Settings;
            self.settings_index = 0;
            self.settings_message.clear();
        }
        for key in &pressed {
            if let KeyCode::Key1 = key { self.slot = 1; }
            if let KeyCode::Key2 = key { self.slot = 2; }
            if let KeyCode::Key3 = key { self.slot = 3; }
            if let KeyCode::Key4 = key { self.slot = 4; }
            if let KeyCode::Key5 = key { self.slot = 5; }
            if let KeyCode::Key6 = key { self.slot = 6; }
        }
        for action in actions {
            match action.as_str() {
                "save" => { self.screen = Screen::SlotSave; self.slot_menu_message.clear(); }
                "load" => { self.screen = Screen::SlotLoad; self.slot_menu_message.clear(); }
                _ => self.menu_primary_action(),
            }
        }
    }

    fn draw_slot_menu(&mut self, title: &str) {
        let pressed: Vec<KeyCode> = get_keys_pressed().into_iter().collect();
        if pressed.contains(&KeyCode::Escape) {
            self.screen = Screen::Menu;
            return;
        }
        if pressed.contains(&KeyCode::Up) || pressed.contains(&KeyCode::W) {
            self.slot = (self.slot - 2) % crate::config::get().save_slots + 1;
        }
        if pressed.contains(&KeyCode::Down) || pressed.contains(&KeyCode::S) {
            self.slot = self.slot % crate::config::get().save_slots + 1;
        }
        for key in &pressed {
            if let KeyCode::Key1 = key { self.slot = 1; }
            if let KeyCode::Key2 = key { self.slot = 2; }
            if let KeyCode::Key3 = key { self.slot = 3; }
            if let KeyCode::Key4 = key { self.slot = 4; }
            if let KeyCode::Key5 = key { self.slot = 5; }
            if let KeyCode::Key6 = key { self.slot = 6; }
        }
        if pressed.contains(&KeyCode::Enter) || pressed.contains(&KeyCode::Space) {
            if title == "SAVE GAME" {
                self.save_game();
            } else {
                self.load_game();
            }
        }
        let (mx, my) = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left) {
            let (left, _, centers) = self.slot_layout();
            let mut hit = false;
            for (index, center) in centers.iter().enumerate() {
                if (my - center).abs() <= 19.0 && mx >= left && mx <= left + 460.0 {
                    self.slot = index + 1;
                    if title == "SAVE GAME" { self.save_game(); } else { self.load_game(); }
                    hit = true;
                    break;
                }
            }
            if !hit {
                self.screen = Screen::Menu;
            }
        }

        let (left, top, centers) = self.slot_layout();
        let right = left + 460.0;
        let bottom = centers.last().copied().unwrap_or(top) + 19.0;
        draw_rectangle(left - 20.0, top - 20.0, right + 20.0 - (left - 20.0), bottom + 50.0 - (top - 20.0), PANEL_COLOR);
        draw_rectangle_lines(left - 20.0, top - 20.0, right + 20.0 - (left - 20.0), bottom + 50.0 - (top - 20.0), 2.0, INV_BORDER_COLOR);
        draw_text_at(title, screen_width() / 2.0, top + 24.0, 24, INV_ACCENT_COLOR, &self.assets.font, 0.5, 0.5);
        for (index, center) in centers.iter().enumerate() {
            let slot = index + 1;
            let selected = slot == self.slot;
            draw_rectangle(left, center - 19.0, 460.0, 38.0,
                if selected { SLOT_SELECTED_COLOR } else { SLOT_COLOR });
            draw_rectangle_lines(left, center - 19.0, 460.0, 38.0,
                if selected { 2.0 } else { 1.0 },
                if selected { INV_SELECTED_BORDER } else { INV_SLOT_BORDER });
            let detail = match save::load(&self.assets, slot) {
                Some(state) => format!(
                    "{}  -  {}g  -  {}",
                    std::path::Path::new(&state.map).file_stem().unwrap_or_default().to_string_lossy(),
                    state.gold, state.timestamp
                ),
                None => "EMPTY".into(),
            };
            draw_text_at(&format!("SLOT {slot}   {detail}"), left + 16.0, *center, 13,
                if selected { INV_ACCENT_COLOR } else { TEXT_COLOR }, &self.assets.font, 0.0, 0.5);
        }
        draw_text_at("UP/DOWN or 1-6 select   ENTER confirm   ESC back",
            screen_width() / 2.0, bottom + 26.0, 10, MENU_MUTED_TEXT, &self.assets.font, 0.5, 0.5);
        if !self.slot_menu_message.is_empty() {
            draw_text_at(&self.slot_menu_message, screen_width() / 2.0, bottom + 44.0,
                10, MENU_MUTED_TEXT, &self.assets.font, 0.5, 0.5);
        }
    }

    fn slot_layout(&self) -> (f32, f32, Vec<f32>) {
        let rows = crate::config::get().save_slots as f32 * 38.0;
        let rows_top = screen_height() / 2.0 - rows / 2.0;
        let top = rows_top - 48.0;
        let left = (screen_width() - 460.0) / 2.0;
        let centers = (0..crate::config::get().save_slots)
            .map(|i| rows_top + 19.0 + i as f32 * 38.0)
            .collect();
        (left, top, centers)
    }

    fn draw_end_screen(&self, title: &str, message: &str, hint: &str) {
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), config::MENU_BACKGROUND);
        draw_text_at(title, screen_width() / 2.0, screen_height() / 2.0 - 70.0,
            crate::config::get().death_font_size, TEXT_COLOR, &self.assets.font, 0.5, 0.5);
        draw_text_at(message, screen_width() / 2.0, screen_height() / 2.0 - 18.0,
            crate::config::get().death_message_font_size, TEXT_COLOR, &self.assets.font, 0.5, 0.5);
        draw_text_at(hint, screen_width() / 2.0, screen_height() / 2.0 + 32.0,
            crate::config::get().death_hint_font_size, MENU_MUTED_TEXT, &self.assets.font, 0.5, 0.5);
    }
}

fn blocked_tiles_for(map: &MapData, door_unlocked: bool) -> HashSet<Tile> {
    let mut blocked = HashSet::new();
    for prefix in ["chest", "fakeChest", "fountain", "switch", "pressurePlate"] {
        for obj in map.objects_named_prefix(prefix) {
            blocked.insert(map.tile_of(vec2(obj.x, obj.y)));
        }
    }
    if !door_unlocked {
        if let Some(door) = map.object_named("door") {
            blocked.insert(map.tile_of(vec2(door.x, door.y)));
        }
    }
    blocked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chest_collision_survives_save_state_restore() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("maps")
            .join("map3.tmx");
        let map = MapData::load(&path);
        let blocked = blocked_tiles_for(&map, false);
        assert!(blocked.contains(&Tile(15, 13)));
        assert!(blocked.contains(&Tile(16, 9)));
    }

    #[test]
    fn map_transition_swaps_when_fade_out_finishes() {
        let mut transition = Some(MapTransition::FadeOut {
            elapsed: 0.0,
            target: 2,
        });
        assert_eq!(Game::advance_transition(&mut transition, 0.4, 1.0), None);
        assert_eq!(
            transition,
            Some(MapTransition::FadeOut {
                elapsed: 0.4,
                target: 2,
            })
        );
        assert_eq!(Game::advance_transition(&mut transition, 0.6, 1.0), Some(2));
        assert_eq!(transition, Some(MapTransition::FadeIn { elapsed: 0.0 }));

        assert_eq!(Game::advance_transition(&mut transition, 0.5, 1.0), None);
        assert_eq!(transition, Some(MapTransition::FadeIn { elapsed: 0.5 }));
        assert_eq!(Game::advance_transition(&mut transition, 0.5, 1.0), None);
        assert_eq!(transition, None);
    }
}
