use std::path::Path;
use std::sync::OnceLock;

use macroquad::prelude::*;

pub const VERSION: &str = "0.0.1-1";
pub const WINDOW_TITLE: &str = "Untitled RPG";
pub const WINDOW_WIDTH: i32 = 1280;
pub const WINDOW_HEIGHT: i32 = 720;
pub const MAP_FILES: [&str; 5] = [
    "map1.tmx",
    "map2.tmx",
    "map3.tmx",
    "map4.tmx",
    "map5.tmx",
];
pub const SAVE_SLOTS: usize = 6;

pub const TILE_SIZE: f32 = 32.0;
pub const CAMERA_ZOOM: f32 = 3.0;
pub const PLAYER_SPEED: f32 = 160.0;
pub const PLAYER_MAX_HP: i32 = 100;
pub const HEALTH_PER_HEART: i32 = 10;
pub const INTERACTION_DISTANCE: f32 = 40.0;
pub const BASE_ATTACK: i32 = 1;
pub const BASE_DEFENSE: i32 = 1;

pub const SWING_ANGLE_DEGREES: f32 = 5.0;
pub const SWING_SPEED: f32 = 10.0;
pub const SWING_EASE: f32 = 15.0;
pub const FLIP_DURATION: f32 = 0.12;
pub const DAMAGE_FLASH_DURATION: f32 = 0.25;
pub const DAMAGE_NUMBER_LIFETIME: f32 = 0.9;
pub const DAMAGE_NUMBER_RISE: f32 = 55.0;
pub const ENEMY_DEATH_DURATION: f32 = 0.45;
pub const DEATH_ANIMATION_DURATION: f32 = 0.6;
pub const DEATH_SCREEN_DELAY: f32 = 5.0;
pub const DEATH_TITLE: &str = "YOU DIED";
pub const DEATH_MESSAGE: &str = "Your journey ends here.";
pub const DEATH_HINT: &str = "Press ENTER or ESC to return to the menu.";
pub const WIN_TITLE: &str = "YOU ESCAPED!";
pub const WIN_MESSAGE: &str = "The key turned, and the dungeon door opened.";
pub const WIN_HINT: &str = "Press ENTER or ESC to return to the menu.";

pub const DARKNESS_ALPHA: u8 = 245;
pub const ROOM_LIGHT_SPEED: f32 = 2.5;
pub const ROOM_LIGHT_BOOST: f32 = 2.0;
pub const ROOM_FADE_TILES: i32 = 4;
pub const PLAYER_LIGHT_RADIUS_TILES: f32 = 2.0;
pub const PLAYER_LIGHT_PEAK: f32 = 0.25;
pub const ROOM_DIM: f32 = 0.2;

pub const LIGHT_SOURCE_TILE_ID: u32 = 12;
pub const LIGHT_RADIUS_TILES: f32 = 6.0;
pub const LIGHT_RAY_COUNT: usize = 512;
pub const PLAYER_RAY_COUNT: usize = 9;
pub const LIGHT_PEAK: f32 = 0.55;
pub const LIGHTMAP_SAMPLES_PER_TILE: f32 = 4.0;
pub const PLAYER_VISION_TILES: f32 = 2.2;
pub const NONROOM_PLAYER_LIGHT_PEAK: f32 = 0.9;
pub const PLAYER_LIGHT_CONE_DEGREES: f32 = 80.0;
pub const PLAYER_LIGHT_INNER_TILES: f32 = 0.3;
pub const PLAYER_LIGHT_EDGE_FRACTION: f32 = 0.25;
pub const LIGHT_SOURCE_TINT_RADIUS_TILES: f32 = 1.4;
pub const LIGHT_SOURCE_TINT_ALPHA: u8 = 55;
pub const LIGHT_SOURCE_TINT_COLOR: Color = Color::new(1.0, 130.0 / 255.0, 40.0 / 255.0, 1.0);
pub const DAMAGE_NUMBER_FONT_SIZE: u16 = 18;
pub const DEATH_FONT_SIZE: u16 = 32;
pub const DEATH_MESSAGE_FONT_SIZE: u16 = 14;
pub const DEATH_HINT_FONT_SIZE: u16 = 11;
pub const TITLE_SCALE: f32 = 2.0;
pub const MIN_WINDOW_WIDTH: i32 = 960;
pub const MIN_WINDOW_HEIGHT: i32 = 540;

pub const COLUMNS: usize = 4;
pub const ROWS: usize = 3;
pub const SLOT_SIZE: f32 = 64.0;
pub const SLOT_GAP: f32 = 10.0;
pub const MAX_STACK: i32 = 16;
pub const STARTING_GOLD: i32 = 0;
pub const EQUIPMENT_SLOTS: [&str; 3] = ["weapon", "offhand", "armor"];
pub const EQUIPMENT_SLOT_SIZE: f32 = 80.0;
pub const EQUIPMENT_SLOT_GAP: f32 = 10.0;
pub const PANEL_PADDING: f32 = 24.0;
pub const PANEL_GAP: f32 = 28.0;
pub const LABEL_HEIGHT: f32 = 34.0;
pub const STAR_SIZE: f32 = 9.0;
pub const STAR_GAP: f32 = 3.0;

pub const ATTACK_EFFECT_DURATION: f32 = 0.15;
pub const ATTACK_EFFECT_SIZE: f32 = 32.0;
pub const TRANSITION_DURATION: f32 = 1.0;
pub const FOUNTAIN_HEAL: i32 = 20;
pub const FOUNTAIN_COOLDOWN: f32 = 5.0;
pub const SPIKE_DAMAGE: i32 = 10;
pub const SPIKE_COOLDOWN: f32 = 0.9;
pub const ENEMY_DAMAGE_COOLDOWN: f32 = 0.75;

pub const PANEL_COLOR: Color = Color::new(22.0 / 255.0, 26.0 / 255.0, 42.0 / 255.0, 250.0 / 255.0);
pub const SLOT_COLOR: Color = Color::new(44.0 / 255.0, 50.0 / 255.0, 72.0 / 255.0, 1.0);
pub const SLOT_SELECTED_COLOR: Color = Color::new(108.0 / 255.0, 122.0 / 255.0, 168.0 / 255.0, 1.0);
pub const EQUIPMENT_SLOT_BG: Color = Color::new(60.0 / 255.0, 68.0 / 255.0, 96.0 / 255.0, 1.0);
pub const INV_DIM_COLOR: Color = Color::new(4.0 / 255.0, 6.0 / 255.0, 14.0 / 255.0, 175.0 / 255.0);
pub const INV_BORDER_COLOR: Color = Color::new(118.0 / 255.0, 132.0 / 255.0, 180.0 / 255.0, 1.0);
pub const INV_HEADER_COLOR: Color = Color::new(38.0 / 255.0, 44.0 / 255.0, 68.0 / 255.0, 1.0);
pub const INV_ACCENT_COLOR: Color = Color::new(238.0 / 255.0, 188.0 / 255.0, 92.0 / 255.0, 1.0);
pub const INV_SLOT_BORDER: Color = Color::new(78.0 / 255.0, 88.0 / 255.0, 122.0 / 255.0, 1.0);
pub const INV_SELECTED_BORDER: Color = Color::new(246.0 / 255.0, 202.0 / 255.0, 104.0 / 255.0, 1.0);
pub const TEXT_COLOR: Color = Color::new(235.0 / 255.0, 232.0 / 255.0, 213.0 / 255.0, 1.0);
pub const MENU_MUTED_TEXT: Color = Color::new(142.0 / 255.0, 151.0 / 255.0, 164.0 / 255.0, 1.0);
pub const MENU_BACKGROUND: Color = Color::new(10.0 / 255.0, 12.0 / 255.0, 20.0 / 255.0, 1.0);

static RUNTIME: OnceLock<RuntimeConfig> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub version: String,
    pub window_title: String,
    pub window_width: i32,
    pub window_height: i32,
    pub save_slots: usize,
    pub camera_zoom: f32,
    pub player_speed: f32,
    pub player_max_hp: i32,
    pub health_per_heart: i32,
    pub interaction_distance: f32,
    pub base_attack: i32,
    pub base_defense: i32,
    pub swing_angle_degrees: f32,
    pub swing_speed: f32,
    pub swing_ease: f32,
    pub flip_duration: f32,
    pub damage_flash_duration: f32,
    pub damage_number_lifetime: f32,
    pub damage_number_rise: f32,
    pub enemy_death_duration: f32,
    pub death_animation_duration: f32,
    pub death_screen_delay: f32,
    pub darkness_alpha: u8,
    pub room_light_speed: f32,
    pub room_light_boost: f32,
    pub room_fade_tiles: i32,
    pub player_light_radius_tiles: f32,
    pub player_light_peak: f32,
    pub room_dim: f32,
    pub light_radius_tiles: f32,
    pub light_ray_count: usize,
    pub player_ray_count: usize,
    pub light_peak: f32,
    pub lightmap_samples_per_tile: f32,
    pub player_vision_tiles: f32,
    pub nonroom_player_light_peak: f32,
    pub player_light_cone_degrees: f32,
    pub player_light_inner_tiles: f32,
    pub player_light_edge_fraction: f32,
    pub light_source_tint_radius_tiles: f32,
    pub light_source_tint_alpha: u8,
    pub damage_number_font_size: u16,
    pub death_font_size: u16,
    pub death_message_font_size: u16,
    pub death_hint_font_size: u16,
    pub title_scale: f32,
    pub min_window_width: i32,
    pub min_window_height: i32,
    pub columns: usize,
    pub rows: usize,
    pub slot_size: f32,
    pub slot_gap: f32,
    pub max_stack: i32,
    pub starting_gold: i32,
    pub equipment_slot_size: f32,
    pub equipment_slot_gap: f32,
    pub panel_padding: f32,
    pub panel_gap: f32,
    pub label_height: f32,
    pub star_size: f32,
    pub star_gap: f32,
    pub attack_effect_duration: f32,
    pub attack_effect_size: f32,
    pub transition_duration: f32,
    pub fountain_heal: i32,
    pub fountain_cooldown: f32,
    pub spike_damage: i32,
    pub spike_cooldown: f32,
    pub enemy_damage_cooldown: f32,
    pub death_title: String,
    pub death_message: String,
    pub death_hint: String,
    pub win_title: String,
    pub win_message: String,
    pub win_hint: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            version: VERSION.into(),
            window_title: WINDOW_TITLE.into(),
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
            save_slots: SAVE_SLOTS,
            camera_zoom: CAMERA_ZOOM,
            player_speed: PLAYER_SPEED,
            player_max_hp: PLAYER_MAX_HP,
            health_per_heart: HEALTH_PER_HEART,
            interaction_distance: INTERACTION_DISTANCE,
            base_attack: BASE_ATTACK,
            base_defense: BASE_DEFENSE,
            swing_angle_degrees: SWING_ANGLE_DEGREES,
            swing_speed: SWING_SPEED,
            swing_ease: SWING_EASE,
            flip_duration: FLIP_DURATION,
            damage_flash_duration: DAMAGE_FLASH_DURATION,
            damage_number_lifetime: DAMAGE_NUMBER_LIFETIME,
            damage_number_rise: DAMAGE_NUMBER_RISE,
            enemy_death_duration: ENEMY_DEATH_DURATION,
            death_animation_duration: DEATH_ANIMATION_DURATION,
            death_screen_delay: DEATH_SCREEN_DELAY,
            darkness_alpha: DARKNESS_ALPHA,
            room_light_speed: ROOM_LIGHT_SPEED,
            room_light_boost: ROOM_LIGHT_BOOST,
            room_fade_tiles: ROOM_FADE_TILES,
            player_light_radius_tiles: PLAYER_LIGHT_RADIUS_TILES,
            player_light_peak: PLAYER_LIGHT_PEAK,
            room_dim: ROOM_DIM,
            light_radius_tiles: LIGHT_RADIUS_TILES,
            light_ray_count: LIGHT_RAY_COUNT,
            player_ray_count: PLAYER_RAY_COUNT,
            light_peak: LIGHT_PEAK,
            lightmap_samples_per_tile: LIGHTMAP_SAMPLES_PER_TILE,
            player_vision_tiles: PLAYER_VISION_TILES,
            nonroom_player_light_peak: NONROOM_PLAYER_LIGHT_PEAK,
            player_light_cone_degrees: PLAYER_LIGHT_CONE_DEGREES,
            player_light_inner_tiles: PLAYER_LIGHT_INNER_TILES,
            player_light_edge_fraction: PLAYER_LIGHT_EDGE_FRACTION,
            light_source_tint_radius_tiles: LIGHT_SOURCE_TINT_RADIUS_TILES,
            light_source_tint_alpha: LIGHT_SOURCE_TINT_ALPHA,
            damage_number_font_size: DAMAGE_NUMBER_FONT_SIZE,
            death_font_size: DEATH_FONT_SIZE,
            death_message_font_size: DEATH_MESSAGE_FONT_SIZE,
            death_hint_font_size: DEATH_HINT_FONT_SIZE,
            title_scale: TITLE_SCALE,
            min_window_width: MIN_WINDOW_WIDTH,
            min_window_height: MIN_WINDOW_HEIGHT,
            columns: COLUMNS,
            rows: ROWS,
            slot_size: SLOT_SIZE,
            slot_gap: SLOT_GAP,
            max_stack: MAX_STACK,
            starting_gold: STARTING_GOLD,
            equipment_slot_size: EQUIPMENT_SLOT_SIZE,
            equipment_slot_gap: EQUIPMENT_SLOT_GAP,
            panel_padding: PANEL_PADDING,
            panel_gap: PANEL_GAP,
            label_height: LABEL_HEIGHT,
            star_size: STAR_SIZE,
            star_gap: STAR_GAP,
            attack_effect_duration: ATTACK_EFFECT_DURATION,
            attack_effect_size: ATTACK_EFFECT_SIZE,
            transition_duration: TRANSITION_DURATION,
            fountain_heal: FOUNTAIN_HEAL,
            fountain_cooldown: FOUNTAIN_COOLDOWN,
            spike_damage: SPIKE_DAMAGE,
            spike_cooldown: SPIKE_COOLDOWN,
            enemy_damage_cooldown: ENEMY_DAMAGE_COOLDOWN,
            death_title: DEATH_TITLE.into(),
            death_message: DEATH_MESSAGE.into(),
            death_hint: DEATH_HINT.into(),
            win_title: WIN_TITLE.into(),
            win_message: WIN_MESSAGE.into(),
            win_hint: WIN_HINT.into(),
        }
    }
}

pub fn init(path: &Path) {
    let _ = RUNTIME.set(RuntimeConfig::load(path));
}

pub fn get() -> &'static RuntimeConfig {
    RUNTIME.get_or_init(RuntimeConfig::default)
}

impl RuntimeConfig {
    pub fn load(path: &Path) -> Self {
        let mut config = Self::default();
        let Ok(text) = std::fs::read_to_string(path) else {
            return config;
        };
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            config.set(key.trim(), value.trim());
        }
        config
    }

    fn set(&mut self, key: &str, value: &str) {
        match key {
            "version" => self.version = value.into(),
            "window_title" => self.window_title = value.into(),
            "window_width" => self.window_width = value.parse().unwrap_or(self.window_width),
            "window_height" => self.window_height = value.parse().unwrap_or(self.window_height),
            "save_slots" => self.save_slots = value.parse().unwrap_or(self.save_slots),
            "camera_zoom" => self.camera_zoom = value.parse().unwrap_or(self.camera_zoom),
            "player_speed" => self.player_speed = value.parse().unwrap_or(self.player_speed),
            "player_max_hp" => self.player_max_hp = value.parse().unwrap_or(self.player_max_hp),
            "health_per_heart" => {
                self.health_per_heart = value.parse().unwrap_or(self.health_per_heart)
            }
            "interaction_distance" => {
                self.interaction_distance = value.parse().unwrap_or(self.interaction_distance)
            }
            "base_attack" => self.base_attack = value.parse().unwrap_or(self.base_attack),
            "base_defense" => self.base_defense = value.parse().unwrap_or(self.base_defense),
            "swing_angle_degrees" => {
                self.swing_angle_degrees = value.parse().unwrap_or(self.swing_angle_degrees)
            }
            "swing_speed" => self.swing_speed = value.parse().unwrap_or(self.swing_speed),
            "swing_ease" => self.swing_ease = value.parse().unwrap_or(self.swing_ease),
            "flip_duration" => self.flip_duration = value.parse().unwrap_or(self.flip_duration),
            "damage_flash_duration" => {
                self.damage_flash_duration = value.parse().unwrap_or(self.damage_flash_duration)
            }
            "damage_number_lifetime" => {
                self.damage_number_lifetime = value.parse().unwrap_or(self.damage_number_lifetime)
            }
            "damage_number_rise" => {
                self.damage_number_rise = value.parse().unwrap_or(self.damage_number_rise)
            }
            "enemy_death_duration" => {
                self.enemy_death_duration = value.parse().unwrap_or(self.enemy_death_duration)
            }
            "death_animation_duration" => {
                self.death_animation_duration = value.parse().unwrap_or(self.death_animation_duration)
            }
            "death_screen_delay" => {
                self.death_screen_delay = value.parse().unwrap_or(self.death_screen_delay)
            }
            "darkness_alpha" => {
                self.darkness_alpha = value.parse().unwrap_or(self.darkness_alpha)
            }
            "room_light_speed" => {
                self.room_light_speed = value.parse().unwrap_or(self.room_light_speed)
            }
            "room_light_boost" => {
                self.room_light_boost = value.parse().unwrap_or(self.room_light_boost)
            }
            "room_fade_tiles" => {
                self.room_fade_tiles = value.parse().unwrap_or(self.room_fade_tiles)
            }
            "player_light_radius_tiles" => {
                self.player_light_radius_tiles = value
                    .parse()
                    .unwrap_or(self.player_light_radius_tiles)
            }
            "player_light_peak" => {
                self.player_light_peak = value.parse().unwrap_or(self.player_light_peak)
            }
            "room_dim" => self.room_dim = value.parse().unwrap_or(self.room_dim),
            "light_radius_tiles" => self.light_radius_tiles = value.parse().unwrap_or(self.light_radius_tiles),
            "light_ray_count" => self.light_ray_count = value.parse().unwrap_or(self.light_ray_count),
            "player_ray_count" => self.player_ray_count = value.parse().unwrap_or(self.player_ray_count),
            "light_peak" => self.light_peak = value.parse().unwrap_or(self.light_peak),
            "lightmap_samples_per_tile" => {
                self.lightmap_samples_per_tile = value.parse().unwrap_or(self.lightmap_samples_per_tile)
            }
            "player_vision_tiles" => self.player_vision_tiles = value.parse().unwrap_or(self.player_vision_tiles),
            "nonroom_player_light_peak" => {
                self.nonroom_player_light_peak = value.parse().unwrap_or(self.nonroom_player_light_peak)
            }
            "player_light_cone_degrees" => {
                self.player_light_cone_degrees = value.parse().unwrap_or(self.player_light_cone_degrees)
            }
            "player_light_inner_tiles" => {
                self.player_light_inner_tiles = value.parse().unwrap_or(self.player_light_inner_tiles)
            }
            "player_light_edge_fraction" => {
                self.player_light_edge_fraction = value.parse().unwrap_or(self.player_light_edge_fraction)
            }
            "light_source_tint_radius_tiles" => {
                self.light_source_tint_radius_tiles = value.parse().unwrap_or(self.light_source_tint_radius_tiles)
            }
            "light_source_tint_alpha" => {
                self.light_source_tint_alpha = value.parse().unwrap_or(self.light_source_tint_alpha)
            }
            "damage_number_font_size" => {
                self.damage_number_font_size = value.parse().unwrap_or(self.damage_number_font_size)
            }
            "death_font_size" => self.death_font_size = value.parse().unwrap_or(self.death_font_size),
            "death_message_font_size" => {
                self.death_message_font_size = value.parse().unwrap_or(self.death_message_font_size)
            }
            "death_hint_font_size" => self.death_hint_font_size = value.parse().unwrap_or(self.death_hint_font_size),
            "title_scale" => self.title_scale = value.parse().unwrap_or(self.title_scale),
            "min_window_width" => self.min_window_width = value.parse().unwrap_or(self.min_window_width),
            "min_window_height" => self.min_window_height = value.parse().unwrap_or(self.min_window_height),
            "columns" => self.columns = value.parse().unwrap_or(self.columns),
            "rows" => self.rows = value.parse().unwrap_or(self.rows),
            "slot_size" => self.slot_size = value.parse().unwrap_or(self.slot_size),
            "slot_gap" => self.slot_gap = value.parse().unwrap_or(self.slot_gap),
            "max_stack" => self.max_stack = value.parse().unwrap_or(self.max_stack),
            "starting_gold" => self.starting_gold = value.parse().unwrap_or(self.starting_gold),
            "equipment_slot_size" => {
                self.equipment_slot_size = value.parse().unwrap_or(self.equipment_slot_size)
            }
            "equipment_slot_gap" => {
                self.equipment_slot_gap = value.parse().unwrap_or(self.equipment_slot_gap)
            }
            "panel_padding" => self.panel_padding = value.parse().unwrap_or(self.panel_padding),
            "panel_gap" => self.panel_gap = value.parse().unwrap_or(self.panel_gap),
            "label_height" => self.label_height = value.parse().unwrap_or(self.label_height),
            "star_size" => self.star_size = value.parse().unwrap_or(self.star_size),
            "star_gap" => self.star_gap = value.parse().unwrap_or(self.star_gap),
            "attack_effect_duration" => {
                self.attack_effect_duration = value.parse().unwrap_or(self.attack_effect_duration)
            }
            "attack_effect_size" => {
                self.attack_effect_size = value.parse().unwrap_or(self.attack_effect_size)
            }
            "transition_duration" => {
                self.transition_duration = value.parse().unwrap_or(self.transition_duration)
            }
            "fountain_heal" => self.fountain_heal = value.parse().unwrap_or(self.fountain_heal),
            "fountain_cooldown" => {
                self.fountain_cooldown = value.parse().unwrap_or(self.fountain_cooldown)
            }
            "spike_damage" => self.spike_damage = value.parse().unwrap_or(self.spike_damage),
            "spike_cooldown" => self.spike_cooldown = value.parse().unwrap_or(self.spike_cooldown),
            "enemy_damage_cooldown" => {
                self.enemy_damage_cooldown = value.parse().unwrap_or(self.enemy_damage_cooldown)
            }
            "death_title" => self.death_title = value.into(),
            "death_message" => self.death_message = value.into(),
            "death_hint" => self.death_hint = value.into(),
            "win_title" => self.win_title = value.into(),
            "win_message" => self.win_message = value.into(),
            "win_hint" => self.win_hint = value.into(),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_config_parses_overrides() {
        let mut config = RuntimeConfig::default();
        config.set("spike_cooldown", "0.9");
        config.set("death_title", "GAME OVER");
        config.set("light_ray_count", "128");
        config.set("player_ray_count", "7");
        config.set("room_fade_tiles", "6");
        config.set("unknown", "ignored");
        assert_eq!(config.spike_cooldown, 0.9);
        assert_eq!(config.death_title, "GAME OVER");
        assert_eq!(config.spike_damage, SPIKE_DAMAGE);
        assert_eq!(config.light_ray_count, 128);
        assert_eq!(config.player_ray_count, 7);
        assert_eq!(config.room_fade_tiles, 6);
    }
}
