use std::collections::{HashMap, VecDeque};

use macroquad::prelude::*;
use serde::Deserialize;

use crate::assets::Assets;
use crate::config::*;
use crate::map::{log_line, MapData, Tile};

#[derive(Clone, Debug, Deserialize)]
struct MobAnimationJson {
    animation: serde_json::Value,
    #[serde(default = "default_true")]
    looped: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
struct MobJson {
    mob_id: Option<String>,
    textures: Option<String>,
    name: Option<String>,
    speed: Option<f32>,
    damage: Option<i32>,
    damage_speed: Option<f32>,
    #[serde(default)]
    frames: MobFramesJson,
    #[serde(default)]
    animations: HashMap<String, MobAnimationJson>,
}

#[derive(Clone, Debug, Deserialize)]
struct MobFramesJson {
    #[serde(default = "default_frame_size")]
    frame_size: [f32; 2],
    #[serde(default = "default_frame_columns")]
    frame_columns: u32,
    #[serde(default = "default_duration")]
    frame_duration: f32,
}

impl Default for MobFramesJson {
    fn default() -> Self {
        Self {
            frame_size: default_frame_size(),
            frame_columns: default_frame_columns(),
            frame_duration: default_duration(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct EnemyStatsJson {
    id: Option<String>,
    name: Option<String>,
    speed: Option<f32>,
    damage: Option<i32>,
    damage_speed: Option<f32>,
}

fn title_case(value: &str) -> String {
    value
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn default_duration() -> f32 {
    0.1
}

fn default_frame_size() -> [f32; 2] {
    [32.0, 32.0]
}

fn default_frame_columns() -> u32 {
    6
}

#[derive(Clone, Debug)]
struct MobAnimation {
    frames: Vec<usize>,
    looped: bool,
}

#[derive(Clone, Debug)]
pub struct MobType {
    #[allow(dead_code)]
    pub name: String,
    pub texture_name: String,
    pub frame_size: Vec2,
    pub columns: u32,
    pub frame_duration: f32,
    pub speed: f32,
    pub damage: i32,
    pub damage_speed: f32,
    walking: MobAnimation,
    defeat: MobAnimation,
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub mob: String,
    pub pos: Vec2,
    pub target: Vec2,
    pub moving: bool,
    pub facing_x: f32,
    pub swing_phase: f32,
    pub flip_timer: f32,
    pub flip_from: f32,
    pub flip_to: f32,
    pub angle: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub alpha: f32,
    pub frame_index: usize,
    pub frame_timer: f32,
    pub dying: bool,
    pub death_timer: f32,
    pub spawn_idle: f32,
    pub attack_timer: f32,
}

impl Enemy {
    pub fn new(mob: &str, pos: Vec2, spawn_idle: f32) -> Self {
        Self {
            mob: mob.to_string(),
            pos,
            target: pos,
            moving: false,
            spawn_idle: spawn_idle.max(0.0),
            attack_timer: 0.0,
            facing_x: 1.0,
            swing_phase: 0.0,
            flip_timer: 0.0,
            flip_from: 1.0,
            flip_to: 1.0,
            angle: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            alpha: 1.0,
            frame_index: 0,
            frame_timer: 0.0,
            dying: false,
            death_timer: 0.0,
        }
    }

    pub fn die(&mut self) {
        if self.dying {
            return;
        }
        self.dying = true;
        self.death_timer = crate::config::get().enemy_death_duration;
        self.moving = false;
        self.angle = 0.0;
        self.scale_x = 1.0;
        self.scale_y = 1.0;
        self.alpha = 1.0;
    }

    pub fn is_idle(&self) -> bool {
        self.spawn_idle > 0.0
    }

    fn tick_spawn_idle(&mut self, dt: f32) -> bool {
        if self.spawn_idle <= 0.0 {
            return false;
        }
        self.spawn_idle = (self.spawn_idle - dt).max(0.0);
        true
    }

    fn update_motion(&mut self, dt: f32, speed: f32) {
        if !self.moving {
            return;
        }
        let delta = self.target - self.pos;
        let distance = delta.length();
        let step = speed * dt;
        if distance <= step || distance < 0.01 {
            self.pos = self.target;
            self.moving = false;
            return;
        }
        self.pos += delta.normalize() * step;
    }

    fn update_swing(&mut self, dt: f32) {
        let mut target = 0.0;
        if self.moving {
            let sign = if self.facing_x < 0.0 { -1.0 } else { 1.0 };
            target = crate::config::get().swing_angle_degrees
                * sign
                * (0.75 + 0.25 * (self.swing_phase).sin());
            self.swing_phase += dt * crate::config::get().swing_speed;
        }
        let ease = (dt * crate::config::get().swing_ease).min(1.0);
        self.angle += (target - self.angle) * ease;
        if self.angle.abs() < 0.01 {
            self.angle = 0.0;
        }
        if self.flip_timer > 0.0 {
            self.flip_timer = (self.flip_timer - dt).max(0.0);
            let progress = 1.0 - self.flip_timer / crate::config::get().flip_duration;
            self.scale_x = self.flip_from + (self.flip_to - self.flip_from) * progress;
            if self.flip_timer == 0.0 {
                self.scale_x = self.flip_to;
            }
        } else if !self.dying {
            self.scale_x = self.flip_to;
        }
    }

    fn set_facing(&mut self, direction_x: f32) {
        if direction_x == 0.0 {
            return;
        }
        let facing = if direction_x > 0.0 { 1.0 } else { -1.0 };
        if facing != self.facing_x {
            self.flip_from = self.scale_x;
            self.flip_to = facing;
            self.flip_timer = crate::config::get().flip_duration;
            self.facing_x = facing;
            self.swing_phase = 0.0;
        }
    }

    pub fn step_to(&mut self, target: Vec2) {
        self.target = target;
        self.moving = true;
        self.set_facing(target.x - self.pos.x);
    }
}

pub struct EnemyManager {
    pub enemies: Vec<Enemy>,
    pub mobs: HashMap<String, MobType>,
    pub spawn_points: Vec<(f32, f32, String)>,
    pub armed: Vec<bool>,
    pub spawned: Vec<bool>,
    pending: Vec<Option<f32>>,
    disabled: bool,
}

pub fn attack_cooldown(damage_speed: f32, fallback: f32) -> f32 {
    if damage_speed > 0.0 {
        (1.0 / damage_speed).max(0.05)
    } else {
        fallback.max(0.0)
    }
}

fn spawn_log_line(source: &str, id: &str, name: &str, pos: Vec2, index: usize, idle: f32) -> String {
    let tile = Tile(
        (pos.x / TILE_SIZE).floor() as i32,
        (pos.y / TILE_SIZE).floor() as i32,
    );
    format!(
        "[spawn] {source} -> {id} ({name}) #{index} at ({:.0}, {:.0}) tile ({}, {}) idle={idle:.2}s",
        pos.x, pos.y, tile.0, tile.1
    )
}

impl EnemyManager {
    pub fn load(map: &MapData, assets: &Assets) -> Self {
        let mobs = load_mobs(assets);
        let mut spawn_points = Vec::new();
        for obj in map.objects_named_prefix("spawnEnemy") {
            let id = if !obj.user_type.is_empty() {
                obj.user_type.clone()
            } else {
                "bat".into()
            };
            spawn_points.push((obj.x, obj.y, id));
        }
        let len = spawn_points.len();
        Self {
            enemies: Vec::new(),
            mobs,
            spawn_points,
            armed: vec![false; len],
            spawned: vec![false; len],
            pending: vec![None; len],
            disabled: false,
        }
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
        if disabled {
            self.enemies.clear();
            self.clear_pending();
        }
    }

    pub fn reset_triggers(&mut self) {
        self.armed.iter_mut().for_each(|value| *value = false);
        self.spawned.iter_mut().for_each(|value| *value = false);
        self.clear_pending();
    }

    fn clear_pending(&mut self) {
        self.pending.iter_mut().for_each(|value| *value = None);
    }

    pub fn spawn(&mut self, id: &str, pos: Vec2) {
        self.spawn_from("script", id, pos);
    }

    pub fn spawn_from(&mut self, source: &str, id: &str, pos: Vec2) {
        self.spawn_from_with_idle(source, id, pos, crate::config::get().enemy_spawn_idle);
    }

    pub fn spawn_from_with_idle(&mut self, source: &str, id: &str, pos: Vec2, idle: f32) {
        if self.disabled {
            return;
        }
        let resolved = if self.mobs.contains_key(id) { id } else { "bat" };
        let Some(mob) = self.mobs.get(resolved).cloned() else {
            return;
        };
        let idle = idle.max(0.0);
        self.enemies.push(Enemy::new(resolved, pos, idle));
        let index = self.enemies.len();
        log_line(&spawn_log_line(source, resolved, &mob.name, pos, index, idle));
    }

    pub fn spawn_at_marker(&mut self, source: &str, id: Option<&str>, near: Option<Vec2>) -> bool {
        if self.disabled || self.spawn_points.is_empty() {
            return false;
        }
        let unspawned: Vec<usize> = (0..self.spawn_points.len())
            .filter(|index| !self.spawned[*index])
            .collect();
        let pool = if unspawned.is_empty() {
            (0..self.spawn_points.len()).collect()
        } else {
            unspawned
        };
        let mut candidates: Vec<usize> = pool
            .iter()
            .copied()
            .filter(|index| id.map_or(true, |wanted| self.spawn_points[*index].2 == wanted))
            .collect();
        if candidates.is_empty() {
            candidates = pool;
        }
        let origin = near.unwrap_or(Vec2::ZERO);
        let index = candidates
            .into_iter()
            .min_by(|left, right| {
                let left_pos = vec2(self.spawn_points[*left].0, self.spawn_points[*left].1);
                let right_pos = vec2(self.spawn_points[*right].0, self.spawn_points[*right].1);
                (left_pos - origin)
                    .length_squared()
                    .total_cmp(&(right_pos - origin).length_squared())
            })
            .expect("spawn marker candidate");
        let (x, y, marker_id) = self.spawn_points[index].clone();
        let enemy_id = id.unwrap_or(&marker_id);
        let source = format!("{source}:spawnEnemy{}", index + 1);
        self.spawn_from(&source, enemy_id, vec2(x, y));
        if !self.spawned[index] {
            self.spawned[index] = true;
        }
        self.armed[index] = false;
        self.pending[index] = None;
        true
    }

    pub fn attack_at(&mut self, pos: Vec2) -> bool {
        if self.disabled {
            return false;
        }
        for enemy in self.enemies.iter_mut() {
            if enemy.dying {
                continue;
            }
            if (enemy.pos.x - pos.x).abs() <= TILE_SIZE
                && (enemy.pos.y - pos.y).abs() <= TILE_SIZE
            {
                enemy.die();
                return true;
            }
        }
        false
    }

    pub fn contact_attack(&self, player_pos: Vec2) -> Option<(usize, i32, f32)> {
        if self.disabled {
            return None;
        }
        let mut best: Option<(usize, i32, f32)> = None;
        for (index, enemy) in self.enemies.iter().enumerate() {
            if enemy.dying
                || enemy.is_idle()
                || enemy.attack_timer > 0.0
                || (enemy.pos.x - player_pos.x).abs() >= TILE_SIZE
                || (enemy.pos.y - player_pos.y).abs() >= TILE_SIZE
            {
                continue;
            }
            let Some(mob) = self.mobs.get(&enemy.mob) else {
                continue;
            };
            let attack = (index, mob.damage, mob.damage_speed);
            let better = match best {
                Some((_, damage, speed)) => {
                    attack.1 > damage || (attack.1 == damage && attack.2 > speed)
                }
                None => true,
            };
            if better {
                best = Some(attack);
            }
        }
        best
    }

    pub fn set_attack_cooldown(&mut self, index: usize, cooldown: f32) {
        if let Some(enemy) = self.enemies.get_mut(index) {
            enemy.attack_timer = cooldown.max(0.0);
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.disabled {
            return;
        }
        let mut finished = Vec::new();
        for (index, enemy) in self.enemies.iter_mut().enumerate() {
            if enemy.dying {
                enemy.death_timer = (enemy.death_timer - dt).max(0.0);
                let progress = 1.0 - enemy.death_timer / crate::config::get().enemy_death_duration;
                let mob = self.mobs.get(&enemy.mob);
                if let Some(mob) = mob {
                    if !mob.defeat.frames.is_empty() {
                        let i = (progress * mob.defeat.frames.len() as f32) as usize;
                        let i = i.min(mob.defeat.frames.len() - 1);
                        enemy.frame_index = mob.defeat.frames[i];
                    }
                }
                enemy.scale_x = 1.0 + 0.25 * (std::f32::consts::PI * progress).sin();
                enemy.scale_y = (1.0 - 0.9 * progress).max(0.05);
                enemy.angle = 80.0 * progress * if enemy.facing_x > 0.0 { 1.0 } else { -1.0 };
                enemy.alpha = 1.0 - progress;
                if enemy.death_timer <= 0.0 {
                    finished.push(index);
                }
                continue;
            }
            enemy.attack_timer = (enemy.attack_timer - dt).max(0.0);
            let was_idle = enemy.is_idle();
            if enemy.tick_spawn_idle(dt) {
                continue;
            }
            if was_idle {
                let tile = Tile(
                    (enemy.pos.x / TILE_SIZE).floor() as i32,
                    (enemy.pos.y / TILE_SIZE).floor() as i32,
                );
                log_line(&format!(
                    "[spawn] enemy #{index} AI active at tile ({}, {})",
                    tile.0, tile.1
                ));
            }
            let speed = self.mobs.get(&enemy.mob).map(|m| m.speed).unwrap_or(160.0);
            let duration = self.mobs.get(&enemy.mob).map(|m| m.frame_duration).unwrap_or(0.1);
            enemy.update_swing(dt);
            enemy.update_motion(dt, speed);
            enemy.frame_timer += dt;
            if enemy.frame_timer >= duration {
                enemy.frame_timer %= duration;
                if let Some(mob) = self.mobs.get(&enemy.mob) {
                    let frames = &mob.walking.frames;
                    if !frames.is_empty() {
                        enemy.frame_index += 1;
                        if mob.walking.looped {
                            enemy.frame_index %= frames.len();
                        } else {
                            enemy.frame_index = enemy.frame_index.min(frames.len() - 1);
                        }
                        enemy.frame_index = frames[enemy.frame_index];
                    }
                }
            }
        }
        for index in finished.into_iter().rev() {
            self.enemies.remove(index);
        }
    }

    pub fn step_toward_player(&mut self, player_pos: Vec2, map: &MapData) {
        if self.disabled {
            return;
        }
        let player_tile = map.tile_of(player_pos);
        let width = map.width.max(0) as usize;
        let height = map.height.max(0) as usize;
        if width == 0 || height == 0 || !map.in_bounds(player_tile) {
            return;
        }

        let mut distance = vec![vec![-1i32; width]; height];
        let mut queue = VecDeque::new();
        distance[player_tile.1 as usize][player_tile.0 as usize] = 0;
        queue.push_back(player_tile);
        while let Some(tile) = queue.pop_front() {
            let current = distance[tile.1 as usize][tile.0 as usize];
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let next = Tile(tile.0 + dx, tile.1 + dy);
                if !map.in_bounds(next) || distance[next.1 as usize][next.0 as usize] >= 0 {
                    continue;
                }
                if map.is_blocked(MapData::center_of(next)) {
                    continue;
                }
                distance[next.1 as usize][next.0 as usize] = current + 1;
                queue.push_back(next);
            }
        }

        for enemy in self.enemies.iter_mut() {
            if enemy.dying || enemy.moving || enemy.is_idle() {
                continue;
            }
            let from = map.tile_of(enemy.pos);
            if !map.in_bounds(from) {
                continue;
            }
            let current = distance[from.1 as usize][from.0 as usize];
            let mut best: Option<(i32, Tile)> = None;
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let next = Tile(from.0 + dx, from.1 + dy);
                if !map.in_bounds(next) {
                    continue;
                }
                let candidate = distance[next.1 as usize][next.0 as usize];
                if candidate < 0 || (current >= 0 && candidate >= current) {
                    continue;
                }
                if best.map_or(true, |(value, _)| candidate < value) {
                    best = Some((candidate, next));
                }
            }
            if let Some((_, next)) = best {
                enemy.step_to(MapData::center_of(next));
            }
        }
    }

    pub fn update_triggers(&mut self, dt: f32, player_tile: Tile, map: &MapData) {
        if self.disabled {
            return;
        }
        let delay = crate::config::get().enemy_spawn_delay;
        for index in 0..self.spawn_points.len() {
            if self.spawned[index] {
                continue;
            }
            let (x, y, id) = self.spawn_points[index].clone();
            let marker = format!("spawnEnemy{}", index + 1);
            let tile = map.tile_of(vec2(x, y));
            if player_tile == tile {
                if !self.armed[index] {
                    log_line(&format!(
                        "[spawn] armed {marker} at tile ({}, {})",
                        tile.0, tile.1
                    ));
                }
                self.armed[index] = true;
                self.pending[index] = None;
            } else if self.armed[index] {
                self.armed[index] = false;
                self.pending[index] = Some(delay);
                log_line(&format!(
                    "[spawn] {marker} left tile ({}, {}); spawning in {delay:.2}s",
                    tile.0, tile.1
                ));
            } else if let Some(timer) = self.pending[index].as_mut() {
                *timer -= dt;
                if *timer <= 0.0 {
                    self.pending[index] = None;
                    self.spawned[index] = true;
                    self.spawn_from(&format!("trigger:{marker}"), &id, vec2(x, y));
                }
            }
        }
    }

    pub fn draw(&self, assets: &Assets, visible: Option<&std::collections::HashSet<Tile>>, map: &MapData) {
        if self.disabled {
            return;
        }
        for enemy in &self.enemies {
            if let Some(visible) = visible {
                if !visible.contains(&map.tile_of(enemy.pos)) {
                    continue;
                }
            }
            let Some(mob) = self.mobs.get(&enemy.mob) else { continue };
            let Some(texture) = assets.mob_textures.get(&mob.texture_name) else { continue };
            let cols = mob.columns.max(1);
            let sx = (enemy.frame_index as u32 % cols) as f32 * mob.frame_size.x;
            let sy = (enemy.frame_index as u32 / cols) as f32 * mob.frame_size.y;
            let w = mob.frame_size.x * enemy.scale_x.abs();
            let h = mob.frame_size.y * enemy.scale_y;
            let alpha = (enemy.alpha * 255.0) as u8;
            draw_texture_ex(
                texture,
                enemy.pos.x - w / 2.0,
                enemy.pos.y - h / 2.0,
                Color::new(1.0, 1.0, 1.0, alpha as f32 / 255.0),
                DrawTextureParams {
                    source: Some(Rect::new(sx, sy, mob.frame_size.x, mob.frame_size.y)),
                    dest_size: Some(vec2(w, h)),
                    rotation: -enemy.angle.to_radians(),
                    flip_x: enemy.scale_x < 0.0,
                    ..Default::default()
                },
            );
        }
    }
}

fn load_enemy_stats(assets: &Assets) -> HashMap<String, EnemyStatsJson> {
    let Ok(text) = std::fs::read_to_string(assets.src().join("data/enemies.json")) else {
        return HashMap::new();
    };
    let Ok(stats) = serde_json::from_str::<HashMap<String, EnemyStatsJson>>(&text) else {
        return HashMap::new();
    };
    stats
        .into_iter()
        .map(|(key, value)| (value.id.clone().unwrap_or(key), value))
        .collect()
}

fn load_mobs(assets: &Assets) -> HashMap<String, MobType> {
    let mut result = HashMap::new();
    let stats = load_enemy_stats(assets);
    let dir = assets.src().join("assets/sprites/enemy");
    let Ok(entries) = std::fs::read_dir(dir) else { return result };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let Ok(json) = serde_json::from_str::<MobJson>(&text) else { continue };
        let id = json.mob_id.clone().unwrap_or_else(|| {
            path.file_stem().unwrap().to_string_lossy().to_string()
        });
        let texture_name = json.textures.clone().unwrap_or_else(|| format!("{id}.png"));
        let Some(texture) = assets.mob_textures.get(&texture_name) else { continue };
        let frame_size = vec2(json.frames.frame_size[0], json.frames.frame_size[1]);
        let rows = (texture.height() / frame_size.y).max(1.0) as usize;
        let frame_count = json.frames.frame_columns as usize * rows;
        let walking = parse_animation(json.animations.get("walking"), frame_count, None);
        let defeat = parse_animation(json.animations.get("defeat"), frame_count, Some(vec![0]));

        let name = json.name.clone()
            .or_else(|| stats.get(&id).and_then(|s| s.name.clone()))
            .unwrap_or_else(|| title_case(&id));
        result.insert(id.clone(), MobType {
            name,
            texture_name,
            frame_size,
            columns: json.frames.frame_columns,
            frame_duration: json.frames.frame_duration,
            speed: json.speed.or_else(|| stats.get(&id).and_then(|s| s.speed)).unwrap_or(160.0),
            damage: json.damage.or_else(|| stats.get(&id).and_then(|s| s.damage)).unwrap_or(0),
            damage_speed: stats
                .get(&id)
                .and_then(|s| s.damage_speed)
                .or(json.damage_speed)
                .unwrap_or(1.0),
            walking,
            defeat,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn empty_map() -> MapData {
        MapData {
            name: "test".into(),
            width: 8,
            height: 8,
            tile_width: 32.0,
            tile_height: 32.0,
            tiles: vec![vec![0; 8]; 8],
            flips: vec![vec![0; 8]; 8],
            solid: HashSet::new(),
            spikes: HashSet::new(),
            objects: Vec::new(),
            rooms: HashMap::new(),
            spawn: vec2(48.0, 48.0),
            light_sources: Vec::new(),
            light_blockers: HashSet::new(),
            lightable_tiles: HashSet::new(),
            block_light_tiles: HashSet::new(),
        }
    }

    #[test]
    fn steps_toward_player_on_free_tile() {
        let mut manager = EnemyManager {
            enemies: Vec::new(),
            mobs: HashMap::new(),
            spawn_points: Vec::new(),
            armed: Vec::new(),
            spawned: Vec::new(),
            pending: Vec::new(),
            disabled: false,
        };
        manager.enemies.push(Enemy::new("bat", vec2(112.0, 112.0), 0.0));
        manager.step_toward_player(vec2(224.0, 112.0), &empty_map());
        assert!(manager.enemies[0].moving);
        assert_eq!(manager.enemies[0].target, vec2(144.0, 112.0));
    }

    #[test]
    fn steps_around_walls_toward_player() {
        let mut map = empty_map();
        map.solid.insert(Tile(3, 2));
        map.solid.insert(Tile(2, 3));
        let mut manager = EnemyManager {
            enemies: Vec::new(),
            mobs: HashMap::new(),
            spawn_points: Vec::new(),
            armed: Vec::new(),
            spawned: Vec::new(),
            pending: Vec::new(),
            disabled: false,
        };
        manager.enemies.push(Enemy::new("snake", MapData::center_of(Tile(2, 2)), 0.0));
        manager.step_toward_player(MapData::center_of(Tile(5, 5)), &map);
        let enemy = &manager.enemies[0];
        assert!(enemy.moving);
        assert_ne!(enemy.target, enemy.pos);
        assert!(!map.is_blocked(enemy.target));
        assert_ne!(enemy.target, MapData::center_of(Tile(3, 2)));
        assert_ne!(enemy.target, MapData::center_of(Tile(2, 3)));
    }

    #[test]
    fn trigger_arms_from_target_tile() {
        let mut manager = EnemyManager {
            enemies: Vec::new(),
            mobs: HashMap::new(),
            spawn_points: vec![(80.0, 80.0, "bat".into())],
            armed: vec![false],
            spawned: vec![false],
            pending: vec![None],
            disabled: false,
        };
        manager.update_triggers(0.016, Tile(2, 2), &empty_map());
        assert!(manager.armed[0]);
    }

    #[test]
    fn spawn_waits_until_delay_after_player_leaves() {
        let animation = MobAnimation { frames: vec![0], looped: true };
        let mob = MobType {
            name: "Bat".into(),
            texture_name: "bat.png".into(),
            frame_size: vec2(32.0, 32.0),
            columns: 1,
            frame_duration: 0.1,
            speed: 160.0,
            damage: 10,
            damage_speed: 10.0,
            walking: animation.clone(),
            defeat: animation,
        };
        let mut manager = EnemyManager {
            enemies: Vec::new(),
            mobs: HashMap::new(),
            spawn_points: vec![(80.0, 80.0, "bat".into())],
            armed: vec![false],
            spawned: vec![false],
            pending: vec![None],
            disabled: false,
        };
        manager.mobs.insert("bat".into(), mob);

        manager.update_triggers(0.016, Tile(2, 2), &empty_map());
        assert!(manager.armed[0]);
        manager.update_triggers(0.016, Tile(3, 2), &empty_map());
        assert!(manager.enemies.is_empty());
        manager.update_triggers(0.4, Tile(3, 2), &empty_map());
        assert!(manager.enemies.is_empty());
        manager.update_triggers(0.7, Tile(3, 2), &empty_map());
        assert_eq!(manager.enemies.len(), 1);
        assert!(manager.spawned[0]);
    }

    #[test]
    fn spawn_at_marker_uses_marker_tile() {
        let animation = MobAnimation { frames: vec![0], looped: true };
        let mob = MobType {
            name: "Bat".into(),
            texture_name: "bat.png".into(),
            frame_size: vec2(32.0, 32.0),
            columns: 1,
            frame_duration: 0.1,
            speed: 160.0,
            damage: 10,
            damage_speed: 10.0,
            walking: animation.clone(),
            defeat: animation,
        };
        let mut manager = EnemyManager {
            enemies: Vec::new(),
            mobs: HashMap::new(),
            spawn_points: vec![
                (80.0, 80.0, "bat".into()),
                (240.0, 240.0, "bat".into()),
            ],
            armed: vec![false, false],
            spawned: vec![false, false],
            pending: vec![None, None],
            disabled: false,
        };
        manager.mobs.insert("bat".into(), mob);

        assert!(manager.spawn_at_marker("debug", None, Some(vec2(0.0, 0.0))));
        assert_eq!(manager.enemies[0].pos, vec2(80.0, 80.0));
        assert!(manager.spawned[0]);
        assert!(!manager.spawned[1]);

        assert!(manager.spawn_at_marker("debug", None, Some(vec2(240.0, 240.0))));
        assert_eq!(manager.enemies[1].pos, vec2(240.0, 240.0));
        assert!(manager.spawned[1]);
    }

    #[test]
    fn returning_to_trigger_cancels_pending_spawn() {
        let animation = MobAnimation { frames: vec![0], looped: true };
        let mob = MobType {
            name: "Bat".into(),
            texture_name: "bat.png".into(),
            frame_size: vec2(32.0, 32.0),
            columns: 1,
            frame_duration: 0.1,
            speed: 160.0,
            damage: 10,
            damage_speed: 10.0,
            walking: animation.clone(),
            defeat: animation,
        };
        let mut manager = EnemyManager {
            enemies: Vec::new(),
            mobs: HashMap::new(),
            spawn_points: vec![(80.0, 80.0, "bat".into())],
            armed: vec![false],
            spawned: vec![false],
            pending: vec![None],
            disabled: false,
        };
        manager.mobs.insert("bat".into(), mob);

        manager.update_triggers(0.016, Tile(2, 2), &empty_map());
        manager.update_triggers(0.016, Tile(3, 2), &empty_map());
        manager.update_triggers(0.9, Tile(3, 2), &empty_map());
        manager.update_triggers(0.016, Tile(2, 2), &empty_map());
        manager.update_triggers(0.016, Tile(3, 2), &empty_map());
        manager.update_triggers(0.9, Tile(3, 2), &empty_map());
        assert!(manager.enemies.is_empty());
        manager.update_triggers(0.2, Tile(3, 2), &empty_map());
        assert_eq!(manager.enemies.len(), 1);
    }

    #[test]
    fn disabled_manager_blocks_spawns_and_damage() {
        let mut manager = EnemyManager {
            enemies: Vec::new(),
            mobs: HashMap::new(),
            spawn_points: vec![(80.0, 80.0, "bat".into())],
            armed: vec![false],
            spawned: vec![false],
            pending: vec![None],
            disabled: false,
        };
        manager.set_disabled(true);
        manager.spawn("bat", vec2(80.0, 80.0));
        assert!(!manager.spawn_at_marker("debug", None, Some(vec2(80.0, 80.0))));
        manager.update_triggers(0.016, Tile(2, 2), &empty_map());
        assert!(manager.enemies.is_empty());
        assert!(manager.spawned.iter().all(|value| !*value));
        assert_eq!(manager.contact_attack(vec2(80.0, 80.0)), None);
        assert!(!manager.attack_at(vec2(80.0, 80.0)));
        manager.update(0.5);
        manager.step_toward_player(vec2(0.0, 0.0), &empty_map());
        assert!(manager.enemies.is_empty());
    }

    #[test]
    fn parses_damage_speed_stat() {
        let stats: HashMap<String, EnemyStatsJson> = serde_json::from_str(
            r#"{"bat":{"damage":10,"speed":160,"damage_speed":10}}"#,
        )
        .unwrap();
        assert_eq!(stats["bat"].damage_speed, Some(10.0));
    }

    #[test]
    fn contact_attack_includes_damage_speed() {
        let animation = MobAnimation { frames: vec![0], looped: true };
        let mob = MobType {
            name: "Bat".into(),
            texture_name: "bat.png".into(),
            frame_size: vec2(32.0, 32.0),
            columns: 1,
            frame_duration: 0.1,
            speed: 160.0,
            damage: 10,
            damage_speed: 10.0,
            walking: animation.clone(),
            defeat: animation,
        };
        let mut manager = EnemyManager {
            enemies: Vec::new(),
            mobs: HashMap::new(),
            spawn_points: Vec::new(),
            armed: Vec::new(),
            spawned: Vec::new(),
            pending: Vec::new(),
            disabled: false,
        };
        manager.mobs.insert("bat".into(), mob);
        manager.enemies.push(Enemy::new("bat", vec2(48.0, 48.0), 0.0));
        assert_eq!(manager.contact_attack(vec2(64.0, 48.0)), Some((0, 10, 10.0)));
        assert_eq!(manager.contact_attack(vec2(200.0, 48.0)), None);
    }

    #[test]
    fn spawn_log_names_source_and_tile() {
        let line = spawn_log_line(
            "trigger:spawnEnemy3",
            "bat",
            "Bat",
            vec2(400.0, 432.0),
            7,
            2.0,
        );
        assert_eq!(
            line,
            "[spawn] trigger:spawnEnemy3 -> bat (Bat) #7 at (400, 432) tile (12, 13) idle=2.00s"
        );
    }

    fn bat_mob() -> MobType {
        let animation = MobAnimation { frames: vec![0], looped: true };
        MobType {
            name: "Bat".into(),
            texture_name: "bat.png".into(),
            frame_size: vec2(32.0, 32.0),
            columns: 1,
            frame_duration: 0.1,
            speed: 160.0,
            damage: 10,
            damage_speed: 10.0,
            walking: animation.clone(),
            defeat: animation,
        }
    }

    #[test]
    fn spawn_idle_keeps_enemy_on_marker_until_ai_starts() {
        let mut manager = EnemyManager {
            enemies: Vec::new(),
            mobs: HashMap::new(),
            spawn_points: Vec::new(),
            armed: Vec::new(),
            spawned: Vec::new(),
            pending: Vec::new(),
            disabled: false,
        };
        manager.mobs.insert("bat".into(), bat_mob());
        let spawn = vec2(112.0, 112.0);
        manager.spawn_from_with_idle("test", "bat", spawn, 1.0);
        assert!(manager.enemies[0].is_idle());
        assert_eq!(manager.enemies[0].pos, spawn);

        manager.step_toward_player(vec2(240.0, 112.0), &empty_map());
        manager.update(0.5);
        assert_eq!(manager.enemies[0].pos, spawn);
        assert!(!manager.enemies[0].moving);
        assert_eq!(manager.contact_attack(spawn), None);

        manager.update(0.5);
        assert!(!manager.enemies[0].is_idle());
        manager.step_toward_player(vec2(240.0, 112.0), &empty_map());
        manager.update(0.1);
        assert!(manager.enemies[0].pos.x > spawn.x);
    }

    #[test]
    fn attack_cooldown_follows_damage_speed() {
        assert!((attack_cooldown(10.0, 0.75) - 0.1).abs() < 1e-6);
        assert!((attack_cooldown(5.0, 0.75) - 0.2).abs() < 1e-6);
        assert_eq!(attack_cooldown(0.0, 0.75), 0.75);
    }

    #[test]
    fn contact_damage_cooldown_is_per_enemy() {
        let mut manager = EnemyManager {
            enemies: Vec::new(),
            mobs: HashMap::new(),
            spawn_points: Vec::new(),
            armed: Vec::new(),
            spawned: Vec::new(),
            pending: Vec::new(),
            disabled: false,
        };
        manager.mobs.insert("bat".into(), bat_mob());
        manager.spawn_from_with_idle("test", "bat", vec2(112.0, 112.0), 0.0);
        manager.spawn_from_with_idle("test", "bat", vec2(112.0, 112.0), 0.0);
        let pos = vec2(112.0, 112.0);
        assert_eq!(manager.contact_attack(pos), Some((0, 10, 10.0)));
        manager.set_attack_cooldown(0, 0.5);
        assert_eq!(manager.contact_attack(pos), Some((1, 10, 10.0)));
        manager.set_attack_cooldown(1, 0.5);
        assert_eq!(manager.contact_attack(pos), None);
    }
}

fn parse_animation(
    value: Option<&MobAnimationJson>,
    frame_count: usize,
    default: Option<Vec<usize>>,
) -> MobAnimation {
    let Some(value) = value else {
        return MobAnimation {
            frames: default.unwrap_or_else(|| (0..frame_count).collect()),
            looped: true,
        };
    };
    let frames = match &value.animation {
        serde_json::Value::String(s) if s == "default" => {
            default.unwrap_or_else(|| (0..frame_count).collect())
        }
        serde_json::Value::Null => default.unwrap_or_else(|| (0..frame_count).collect()),
        serde_json::Value::Array(values) => values
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as usize))
            .collect(),
        _ => default.unwrap_or_else(|| (0..frame_count).collect()),
    };
    MobAnimation { frames, looped: value.looped }
}
