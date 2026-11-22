use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::assets::Assets;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct PlayerSave {
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub direction: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct StackSave {
    pub item_id: String,
    pub count: i32,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct InventorySave {
    pub slots: Vec<Option<StackSave>>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct EquipmentSave {
    pub slots: HashMap<String, Option<String>>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct ChestSave {
    pub opened: Vec<usize>,
    pub fake_opened: Vec<usize>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct EnemySave {
    pub enemies: Vec<EnemyEntry>,
    pub spawned: Vec<usize>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct EnemyEntry {
    #[serde(rename = "type")]
    pub kind: String,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub hp: i32,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MapObjectSave {
    pub switches: Vec<bool>,
    pub pressure_plates: Vec<bool>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SaveState {
    pub timestamp: String,
    pub map: String,
    pub player: PlayerSave,
    pub inventory: InventorySave,
    pub equipment: EquipmentSave,
    pub gold: i32,
    pub chests: ChestSave,
    pub enemies: EnemySave,
    pub map_objects: MapObjectSave,
    pub script_flags: Vec<String>,
    pub door_unlocked: bool,
    #[serde(default)]
    pub config: crate::config::RuntimeConfig,
}

pub fn slot_path(assets: &Assets, slot: usize) -> std::path::PathBuf {
    assets.src().join("saves").join(format!("slot{slot}.urpgs"))
}

pub fn save(assets: &Assets, slot: usize, state: &SaveState) {
    let path = slot_path(assets, slot);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, text);
    }
}

pub fn load(assets: &Assets, slot: usize) -> Option<SaveState> {
    let path = slot_path(assets, slot);
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}",
        sod / 3600,
        (sod % 3600) / 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_state_round_trip_preserves_runtime_config() {
        let mut config = crate::config::RuntimeConfig::default();
        config.set("enemy_spawn_distance_tiles", "3.5");
        config.set("death_title", "GAME OVER");
        let state = SaveState { config, ..Default::default() };
        let json = serde_json::to_string(&state).unwrap();
        let loaded: SaveState = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.config.enemy_spawn_distance_tiles, 3.5);
        assert_eq!(loaded.config.death_title, "GAME OVER");

        let mut value = serde_json::to_value(&state).unwrap();
        value.as_object_mut().unwrap().remove("config");
        let legacy: SaveState = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.config.enemy_spawn_distance_tiles, 2.0);
    }
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    (y + if m <= 2 { 1 } else { 0 }, m, d)
}
