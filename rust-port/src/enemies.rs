use std::collections::HashMap;

use macroquad::prelude::*;
use serde::Deserialize;

use crate::assets::Assets;
use crate::config::*;
use crate::map::{MapData, Tile};

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
}

impl Enemy {
    pub fn new(mob: &str, pos: Vec2) -> Self {
        Self {
            mob: mob.to_string(),
            pos,
            target: pos,
            moving: false,
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
        }
    }

    pub fn spawn(&mut self, id: &str, pos: Vec2) {
        let resolved = if self.mobs.contains_key(id) { id } else { "bat" };
        if self.mobs.contains_key(resolved) {
            self.enemies.push(Enemy::new(resolved, pos));
        }
    }

    pub fn spawn_near_player(&mut self, player_pos: Vec2, map: &MapData) {
        let offsets = [
            (-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0),
            (-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0),
        ];
        for (dx, dy) in offsets {
            let pos = player_pos + vec2(dx, dy) * TILE_SIZE;
            if map.is_blocked(pos) {
                continue;
            }
            if (pos - player_pos).length() < TILE_SIZE {
                continue;
            }
            self.spawn("bat", pos);
            return;
        }
    }

    pub fn attack_at(&mut self, pos: Vec2) -> bool {
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

    pub fn contact_attack(&self, player_pos: Vec2) -> Option<(i32, f32)> {
        let mut best: Option<(i32, f32)> = None;
        for enemy in &self.enemies {
            if enemy.dying
                || (enemy.pos.x - player_pos.x).abs() >= TILE_SIZE
                || (enemy.pos.y - player_pos.y).abs() >= TILE_SIZE
            {
                continue;
            }
            let Some(mob) = self.mobs.get(&enemy.mob) else {
                continue;
            };
            let attack = (mob.damage, mob.damage_speed);
            let better = match best {
                Some((damage, speed)) => attack.0 > damage || (attack.0 == damage && attack.1 > speed),
                None => true,
            };
            if better {
                best = Some(attack);
            }
        }
        best
    }

    pub fn update(&mut self, dt: f32) {
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
        let player_tile = map.tile_of(player_pos);
        for enemy in self.enemies.iter_mut() {
            if enemy.dying || enemy.moving {
                continue;
            }
            let from = map.tile_of(enemy.pos);
            let dx = player_tile.0 - from.0;
            let dy = player_tile.1 - from.1;
            if dx == 0 && dy == 0 {
                continue;
            }
            let steps = if dx.abs() >= dy.abs() {
                [(dx.signum(), 0), (0, dy.signum())]
            } else {
                [(0, dy.signum()), (dx.signum(), 0)]
            };
            for (sx, sy) in steps {
                if sx == 0 && sy == 0 {
                    continue;
                }
                let next = Tile(from.0 + sx, from.1 + sy);
                let target = MapData::center_of(next);
                if !map.is_blocked(target) {
                    enemy.step_to(target);
                    break;
                }
            }
        }
    }

    pub fn update_triggers(&mut self, player_pos: Vec2, player_tile: Tile, map: &MapData) {
        for index in 0..self.spawn_points.len() {
            if self.spawned[index] {
                continue;
            }
            let (x, y, id) = self.spawn_points[index].clone();
            if map.tile_of(player_pos) == map.tile_of(vec2(x, y)) {
                self.armed[index] = true;
            } else if self.armed[index] {
                self.armed[index] = false;
                self.spawned[index] = true;
                self.spawn(&id, vec2(x, y));
            }
        }
        let _ = player_tile;
    }

    pub fn draw(&self, assets: &Assets, visible: Option<&std::collections::HashSet<Tile>>, map: &MapData) {
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
        };
        manager.enemies.push(Enemy::new("bat", vec2(112.0, 112.0)));
        manager.step_toward_player(vec2(224.0, 112.0), &empty_map());
        assert!(manager.enemies[0].moving);
        assert_eq!(manager.enemies[0].target, vec2(144.0, 112.0));
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
        };
        manager.mobs.insert("bat".into(), mob);
        manager.enemies.push(Enemy::new("bat", vec2(48.0, 48.0)));
        assert_eq!(manager.contact_attack(vec2(64.0, 48.0)), Some((10, 10.0)));
        assert_eq!(manager.contact_attack(vec2(200.0, 48.0)), None);
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
