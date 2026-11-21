use std::collections::HashMap;
use std::path::Path;

use macroquad::prelude::*;
use serde::Deserialize;


#[derive(Clone, Debug, Deserialize)]
struct ItemJson {
    pub name: String,
    pub texture: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub slot: Option<String>,
    #[serde(default)]
    pub stackable: bool,
    #[serde(default)]
    pub damage: i32,
    #[serde(default)]
    pub defense: i32,
    #[serde(default)]
    pub heal: i32,
    #[serde(default)]
    pub price: i32,
    pub removeable: Option<bool>,
    #[serde(default)]
    pub quality: i32,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub texture: String,
    pub item_type: String,
    pub slot: Option<String>,
    pub stackable: bool,
    pub damage: i32,
    pub defense: i32,
    pub heal: i32,
    pub price: i32,
    pub removeable: bool,
    pub quality: i32,
}

pub fn load_items(path: &Path) -> HashMap<String, Item> {
    let text = std::fs::read_to_string(path).expect("items.json");
    let map: HashMap<String, ItemJson> = serde_json::from_str(&text).expect("parse items.json");
    map.into_iter().map(|(id, raw)| (id.clone(), Item {
        name: raw.name,
        texture: raw.texture,
        item_type: raw.item_type,
        slot: raw.slot,
        stackable: raw.stackable,
        damage: raw.damage,
        defense: raw.defense,
        heal: raw.heal,
        price: raw.price,
        removeable: raw.removeable.unwrap_or(id != "key"),
        quality: raw.quality.clamp(0, 3),
    })).collect()
}

#[derive(Clone, Debug)]
pub struct Stack {
    pub item_id: String,
    pub count: i32,
}

#[derive(Clone, Debug, Default)]
pub struct Inventory {
    pub slots: Vec<Option<Stack>>,
    pub gold: i32,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            slots: vec![None; 12],
            gold: crate::config::get().starting_gold,
        }
    }

    pub fn add(&mut self, item_id: &str, count: i32, stackable: bool) -> bool {
        let mut remaining = count;
        if stackable {
            for stack in self.slots.iter_mut().flatten() {
                if stack.item_id != item_id {
                    continue;
                }
                let space = crate::config::get().max_stack - stack.count;
                if space <= 0 {
                    continue;
                }
                let taken = space.min(remaining);
                stack.count += taken;
                remaining -= taken;
                if remaining == 0 {
                    return true;
                }
            }
        }
        for stack in self.slots.iter_mut() {
            if stack.is_some() {
                continue;
            }
            let taken = crate::config::get().max_stack.min(remaining);
            *stack = Some(Stack { item_id: item_id.into(), count: taken });
            remaining -= taken;
            if remaining == 0 {
                return true;
            }
        }
        false
    }

    pub fn remove_item(&mut self, item_id: &str, count: i32) -> bool {
        let mut remaining = count;
        for slot in self.slots.iter_mut() {
            let Some(stack) = slot else { continue };
            if stack.item_id != item_id {
                continue;
            }
            let removed = stack.count.min(remaining);
            stack.count -= removed;
            remaining -= removed;
            if stack.count == 0 {
                *slot = None;
            }
            if remaining == 0 {
                return true;
            }
        }
        false
    }

    pub fn remove_at(&mut self, index: usize, count: i32) -> Option<(String, i32)> {
        let stack = self.slots.get_mut(index)?.as_mut()?;
        let item_id = stack.item_id.clone();
        let removed = stack.count.min(count);
        stack.count -= removed;
        if stack.count == 0 {
            self.slots[index] = None;
        }
        Some((item_id, removed))
    }

    pub fn count(&self, item_id: &str) -> i32 {
        self.slots
            .iter()
            .flatten()
            .filter(|stack| stack.item_id == item_id)
            .map(|stack| stack.count)
            .sum()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Equipment {
    pub slots: HashMap<String, Option<String>>,
}

impl Equipment {
    pub fn new() -> Self {
        let mut slots = HashMap::new();
        for slot in crate::config::EQUIPMENT_SLOTS {
            slots.insert(slot.to_string(), None);
        }
        Self { slots }
    }

    pub fn get(&self, slot: &str) -> Option<&str> {
        self.slots.get(slot).and_then(|v| v.as_deref())
    }

    pub fn equip(&mut self, slot: &str, item_id: &str) -> Option<String> {
        let previous = self.slots.get(slot).cloned().flatten();
        self.slots.insert(slot.to_string(), Some(item_id.to_string()));
        previous
    }

    pub fn unequip(&mut self, slot: &str) -> Option<String> {
        self.slots.insert(slot.to_string(), None).flatten()
    }

    pub fn combat_stats(&self, items: &HashMap<String, Item>) -> (i32, i32) {
        let mut attack = crate::config::get().base_attack;
        let mut defense = crate::config::get().base_defense;
        for item_id in self.slots.values().flatten() {
            if let Some(item) = items.get(item_id) {
                attack += item.damage;
                defense += item.defense;
            }
        }
        (attack, defense)
    }
}

pub fn draw_quality(x: f32, y: f32, quality: i32) {
    for index in 0..3 {
        let cx = x + index as f32 * (crate::config::get().star_size + crate::config::get().star_gap);
        let mut points = Vec::new();
        for i in 0..10 {
            let radius = if i % 2 == 0 {
                crate::config::get().star_size / 2.0
            } else {
                crate::config::get().star_size / 4.0
            };
            let angle = std::f32::consts::PI / 2.0 + i as f32 * std::f32::consts::PI / 5.0;
            points.push(vec2(cx + angle.cos() * radius, y + angle.sin() * radius));
        }
        let center = vec2(cx, y);
        let first = points[0];
        if index < quality {
            for point in points.iter() {
                draw_triangle(center, *point, first, crate::config::INV_ACCENT_COLOR);
            }
            for pair in points.windows(2) {
                draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, 2.0, crate::config::INV_ACCENT_COLOR);
            }
        } else {
            for pair in points.windows(2) {
                draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, 2.0, crate::config::INV_SLOT_BORDER);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_items() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data/items.json");
        let items = load_items(&path);
        assert_eq!(items.len(), 10);
        assert_eq!(items["titanium_sword"].quality, 3);
    }

    #[test]
    fn defaults_match_python_item_handling() {
        let path = std::env::temp_dir().join("untitled-rpg-item-defaults.json");
        std::fs::write(
            &path,
            r#"{
                "key": {"name":"Key","texture":"key.png","type":"key","quality":9},
                "rock": {"name":"Rock","texture":"rock.png","type":"misc","quality":-2}
            }"#,
        )
        .unwrap();
        let items = load_items(&path);
        let _ = std::fs::remove_file(path);
        assert!(!items["key"].stackable);
        assert!(!items["key"].removeable);
        assert!(!items["rock"].stackable);
        assert!(items["rock"].removeable);
        assert_eq!(items["key"].quality, 3);
        assert_eq!(items["rock"].quality, 0);
    }
}
