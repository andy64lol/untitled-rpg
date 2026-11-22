use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

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

pub const CHEST_REWARD_IDS: [&str; 9] = [
    "apple",
    "pear",
    "banana",
    "bronze_sword",
    "iron_sword",
    "rusty_iron_sword",
    "stainless_steel_shield",
    "leather_armour",
    "titanium_sword",
];

pub const SWING_ANGLE_DEGREES: f32 = 5.0;
pub const SWING_SPEED: f32 = 10.0;
pub const SWING_EASE: f32 = 15.0;
pub const FLIP_DURATION: f32 = 0.12;
pub const DAMAGE_FLASH_DURATION: f32 = 0.25;
pub const DAMAGE_NUMBER_LIFETIME: f32 = 0.9;
pub const DAMAGE_NUMBER_RISE: f32 = 55.0;
pub const ENEMY_DEATH_DURATION: f32 = 0.45;
pub const ENEMY_SPAWN_DISTANCE_TILES: f32 = 2.0;
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

static RUNTIME: AtomicPtr<RuntimeConfig> = AtomicPtr::new(ptr::null_mut());
static ORIGINAL: AtomicPtr<RuntimeConfig> = AtomicPtr::new(ptr::null_mut());

fn default_enemy_spawn_distance_tiles() -> f32 {
    ENEMY_SPAWN_DISTANCE_TILES
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
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
    #[serde(default = "default_enemy_spawn_distance_tiles")]
    pub enemy_spawn_distance_tiles: f32,
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
            enemy_spawn_distance_tiles: ENEMY_SPAWN_DISTANCE_TILES,
            death_title: DEATH_TITLE.into(),
            death_message: DEATH_MESSAGE.into(),
            death_hint: DEATH_HINT.into(),
            win_title: WIN_TITLE.into(),
            win_message: WIN_MESSAGE.into(),
            win_hint: WIN_HINT.into(),
        }
    }
}

fn replace(slot: &AtomicPtr<RuntimeConfig>, config: RuntimeConfig) -> &'static RuntimeConfig {
    let raw = Box::into_raw(Box::new(config));
    let old = slot.swap(raw, Ordering::SeqCst);
    debug_assert!(old != raw);
    unsafe { &*raw }
}

fn instance(slot: &AtomicPtr<RuntimeConfig>) -> &'static RuntimeConfig {
    let mut raw = slot.load(Ordering::SeqCst);
    if raw.is_null() {
        let new = Box::into_raw(Box::new(RuntimeConfig::default()));
        match slot.compare_exchange(ptr::null_mut(), new, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => raw = new,
            Err(existing) => {
                unsafe {
                    drop(Box::from_raw(new));
                }
                raw = existing;
            }
        }
    }
    unsafe { &*raw }
}

pub fn init(path: &Path) {
    let config = RuntimeConfig::load(path);
    replace(&RUNTIME, config.clone());
    replace(&ORIGINAL, config);
}

pub fn get() -> &'static RuntimeConfig {
    instance(&RUNTIME)
}

pub fn original() -> &'static RuntimeConfig {
    instance(&ORIGINAL)
}

pub fn snapshot() -> RuntimeConfig {
    get().clone()
}

pub fn restore(config: &RuntimeConfig) {
    replace(&RUNTIME, config.clone());
}

pub fn keys() -> Vec<String> {
    serde_json::to_value(RuntimeConfig::default())
        .ok()
        .and_then(|value| value.as_object().map(|map| map.keys().cloned().collect()))
        .unwrap_or_default()
}

pub fn value(key: &str) -> Option<String> {
    format_field(serde_json::to_value(get()).ok()?.get(key)?)
}

pub fn original_value(key: &str) -> Option<String> {
    format_field(serde_json::to_value(original()).ok()?.get(key)?)
}

fn format_field(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Null => None,
        other => Some(other.to_string()),
    }
}

pub fn modified() -> Vec<(String, String, String)> {
    keys()
        .into_iter()
        .filter_map(|key| {
            let current = value(&key)?;
            let original = original_value(&key)?;
            (current != original).then_some((key, current, original))
        })
        .collect()
}

pub fn set_value(key: &str, value: &str) -> bool {
    let mut config = get().clone();
    if !config.set(key, value) {
        return false;
    }
    replace(&RUNTIME, config);
    true
}

pub fn reset_value(key: &str) -> bool {
    let Some(value) = original_value(key) else {
        return false;
    };
    set_value(key, &value)
}

pub fn reset_all() {
    restore(original());
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

    pub(crate) fn set(&mut self, key: &str, value: &str) -> bool {
        macro_rules! number {
            ($field:ident, $ty:ty) => {
                match value.parse::<$ty>() {
                    Ok(parsed) => {
                        self.$field = parsed;
                        true
                    }
                    Err(_) => false,
                }
            };
        }
        macro_rules! text {
            ($field:ident) => {{
                self.$field = value.to_string();
                true
            }};
        }
        match key {
            "version" => text!(version),
            "window_title" => text!(window_title),
            "window_width" => number!(window_width, i32),
            "window_height" => number!(window_height, i32),
            "save_slots" => number!(save_slots, usize),
            "camera_zoom" => number!(camera_zoom, f32),
            "player_speed" => number!(player_speed, f32),
            "player_max_hp" => number!(player_max_hp, i32),
            "health_per_heart" => number!(health_per_heart, i32),
            "interaction_distance" => number!(interaction_distance, f32),
            "base_attack" => number!(base_attack, i32),
            "base_defense" => number!(base_defense, i32),
            "swing_angle_degrees" => number!(swing_angle_degrees, f32),
            "swing_speed" => number!(swing_speed, f32),
            "swing_ease" => number!(swing_ease, f32),
            "flip_duration" => number!(flip_duration, f32),
            "damage_flash_duration" => number!(damage_flash_duration, f32),
            "damage_number_lifetime" => number!(damage_number_lifetime, f32),
            "damage_number_rise" => number!(damage_number_rise, f32),
            "enemy_death_duration" => number!(enemy_death_duration, f32),
            "death_animation_duration" => number!(death_animation_duration, f32),
            "death_screen_delay" => number!(death_screen_delay, f32),
            "darkness_alpha" => number!(darkness_alpha, u8),
            "room_light_speed" => number!(room_light_speed, f32),
            "room_light_boost" => number!(room_light_boost, f32),
            "room_fade_tiles" => number!(room_fade_tiles, i32),
            "player_light_radius_tiles" => number!(player_light_radius_tiles, f32),
            "player_light_peak" => number!(player_light_peak, f32),
            "room_dim" => number!(room_dim, f32),
            "light_radius_tiles" => number!(light_radius_tiles, f32),
            "light_ray_count" => number!(light_ray_count, usize),
            "player_ray_count" => number!(player_ray_count, usize),
            "light_peak" => number!(light_peak, f32),
            "lightmap_samples_per_tile" => number!(lightmap_samples_per_tile, f32),
            "player_vision_tiles" => number!(player_vision_tiles, f32),
            "nonroom_player_light_peak" => number!(nonroom_player_light_peak, f32),
            "player_light_cone_degrees" => number!(player_light_cone_degrees, f32),
            "player_light_inner_tiles" => number!(player_light_inner_tiles, f32),
            "player_light_edge_fraction" => number!(player_light_edge_fraction, f32),
            "light_source_tint_radius_tiles" => number!(light_source_tint_radius_tiles, f32),
            "light_source_tint_alpha" => number!(light_source_tint_alpha, u8),
            "damage_number_font_size" => number!(damage_number_font_size, u16),
            "death_font_size" => number!(death_font_size, u16),
            "death_message_font_size" => number!(death_message_font_size, u16),
            "death_hint_font_size" => number!(death_hint_font_size, u16),
            "title_scale" => number!(title_scale, f32),
            "min_window_width" => number!(min_window_width, i32),
            "min_window_height" => number!(min_window_height, i32),
            "columns" => number!(columns, usize),
            "rows" => number!(rows, usize),
            "slot_size" => number!(slot_size, f32),
            "slot_gap" => number!(slot_gap, f32),
            "max_stack" => number!(max_stack, i32),
            "starting_gold" => number!(starting_gold, i32),
            "equipment_slot_size" => number!(equipment_slot_size, f32),
            "equipment_slot_gap" => number!(equipment_slot_gap, f32),
            "panel_padding" => number!(panel_padding, f32),
            "panel_gap" => number!(panel_gap, f32),
            "label_height" => number!(label_height, f32),
            "star_size" => number!(star_size, f32),
            "star_gap" => number!(star_gap, f32),
            "attack_effect_duration" => number!(attack_effect_duration, f32),
            "attack_effect_size" => number!(attack_effect_size, f32),
            "transition_duration" => number!(transition_duration, f32),
            "fountain_heal" => number!(fountain_heal, i32),
            "fountain_cooldown" => number!(fountain_cooldown, f32),
            "spike_damage" => number!(spike_damage, i32),
            "spike_cooldown" => number!(spike_cooldown, f32),
            "enemy_damage_cooldown" => number!(enemy_damage_cooldown, f32),
            "enemy_spawn_distance_tiles" => number!(enemy_spawn_distance_tiles, f32),
            "death_title" => text!(death_title),
            "death_message" => text!(death_message),
            "death_hint" => text!(death_hint),
            "win_title" => text!(win_title),
            "win_message" => text!(win_message),
            "win_hint" => text!(win_hint),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_config_set_rejects_invalid_values() {
        let mut config = RuntimeConfig::default();
        assert!(config.set("enemy_spawn_distance_tiles", "3.5"));
        assert!(!config.set("enemy_spawn_distance_tiles", "soon"));
        assert!(config.set("death_title", "GAME OVER"));
        assert!(!config.set("missing_key", "1"));
        assert_eq!(config.enemy_spawn_distance_tiles, 3.5);
        assert_eq!(config.death_title, "GAME OVER");
    }

    #[test]
    fn runtime_config_parses_overrides() {
        let mut config = RuntimeConfig::default();
        config.set("spike_cooldown", "0.9");
        config.set("enemy_spawn_distance_tiles", "3.5");
        config.set("death_title", "GAME OVER");
        config.set("light_ray_count", "128");
        config.set("player_ray_count", "7");
        config.set("room_fade_tiles", "6");
        config.set("unknown", "ignored");
        assert_eq!(config.spike_cooldown, 0.9);
        assert_eq!(config.enemy_spawn_distance_tiles, 3.5);
        assert_eq!(config.death_title, "GAME OVER");
        assert_eq!(config.spike_damage, SPIKE_DAMAGE);
        assert_eq!(config.light_ray_count, 128);
        assert_eq!(config.player_ray_count, 7);
        assert_eq!(config.room_fade_tiles, 6);
    }
}
