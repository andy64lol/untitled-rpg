use std::collections::{HashMap, HashSet};
use std::path::Path;

use macroquad::prelude::*;
use tiled::{LayerType, Loader};

use crate::config::{TILE_SIZE, LIGHT_SOURCE_TILE_ID};
use crate::LOG;

#[derive(Clone, Debug)]
pub struct MapObject {
    pub x: f32,
    pub y: f32,
    pub name: String,
    pub user_type: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Tile(pub i32, pub i32);

pub struct MapData {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub tile_width: f32,
    pub tile_height: f32,
    pub tiles: Vec<Vec<u32>>,
    pub flips: Vec<Vec<u8>>,
    pub solid: HashSet<Tile>,
    pub spikes: HashSet<Tile>,
    pub objects: Vec<MapObject>,
    pub rooms: HashMap<String, HashSet<Tile>>,
    pub spawn: Vec2,
    pub light_sources: Vec<Vec2>,
    pub light_blockers: HashSet<Tile>,
    pub lightable_tiles: HashSet<Tile>,
    pub block_light_tiles: HashSet<Tile>,
}

impl MapData {
    pub fn load(path: &Path) -> Self {
        let mut loader = Loader::new();
        let map = loader
            .load_tmx_map(path)
            .unwrap_or_else(|e| panic!("failed to load {}: {e}", path.display()));
        let width = map.width as i32;
        let height = map.height as i32;
        let ts = map.tile_width as f32;
        let th = map.tile_height as f32;

        let mut tiles = vec![vec![0u32; width as usize]; height as usize];
        let mut flips = vec![vec![0u8; width as usize]; height as usize];
        let mut light_sources = Vec::new();
        let mut solid = HashSet::new();
        let mut spike_set = HashSet::new();
        let mut rooms: HashMap<String, HashSet<Tile>> = HashMap::new();
        let mut objects = Vec::new();
        let mut spawn = vec2(width as f32 * ts / 2.0, height as f32 * ts / 2.0);

        for layer in map.layers() {
            collect_layer(
                &layer,
                width,
                height,
                &mut tiles,
                &mut flips,
                &mut light_sources,
                &mut solid,
                &mut spike_set,
                &mut rooms,
                &mut objects,
                &mut spawn,
                ts,
                th,
            );
        }

        let transparent: HashSet<Tile> = objects
            .iter()
            .filter(|object| {
                object.name.starts_with("fountain")
                    || object.name.starts_with("chest")
                    || object.name.starts_with("fakeChest")
            })
            .map(|object| object_tile(object.x, object.y, ts, th))
            .chain(spike_set.iter().copied())
            .collect();
        let light_blockers: HashSet<Tile> = solid.difference(&transparent).copied().collect();
        let block_light_tiles: HashSet<Tile> = objects
            .iter()
            .filter(|object| object.name == "block-light")
            .map(|object| object_tile(object.x, object.y, ts, th))
            .collect();
        let lightable_tiles: HashSet<Tile> = objects
            .iter()
            .filter(|object| object.name == "lightable-tile")
            .map(|object| object_tile(object.x, object.y, ts, th))
            .filter(|tile| solid.contains(tile))
            .collect();
        for object in objects
            .iter()
            .filter(|object| object.name == "light" || object.user_type.eq_ignore_ascii_case("light"))
        {
            light_sources.push(vec2(object.x, object.y));
        }

        Self {
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            width,
            height,
            tile_width: ts,
            tile_height: th,
            tiles,
            flips,
            solid,
            spikes: spike_set,
            objects,
            rooms,
            spawn,
            light_sources,
            light_blockers,
            lightable_tiles,
            block_light_tiles,
        }
    }

    pub fn has_rooms(&self) -> bool {
        !self.rooms.is_empty()
    }

    pub fn tile_of(&self, pos: Vec2) -> Tile {
        Tile(
            (pos.x / self.tile_width).floor() as i32,
            (pos.y / self.tile_height).floor() as i32,
        )
    }

    pub fn center_of(tile: Tile) -> Vec2 {
        vec2(
            tile.0 as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            tile.1 as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        )
    }

    pub fn is_solid(&self, tile: Tile) -> bool {
        self.solid.contains(&tile)
    }

    pub fn player_light_blockers(&self) -> HashSet<Tile> {
        self.light_blockers
            .iter()
            .chain(self.block_light_tiles.iter())
            .copied()
            .collect()
    }

    pub fn lightable_for_lights(&self) -> HashSet<Tile> {
        self.lightable_tiles
            .difference(&self.block_light_tiles)
            .copied()
            .collect()
    }

    pub fn is_blocked(&self, pos: Vec2) -> bool {
        let tile = self.tile_of(pos);
        if !self.in_bounds(tile) || self.is_solid(tile) {
            return true;
        }
        self.objects.iter().any(|object| {
            let collider = object.name.starts_with("chest")
                || object.name.starts_with("fakeChest")
                || object.name.starts_with("fountain")
                || object.name.starts_with("switch")
                || object.name.starts_with("pressurePlate")
                || object.name.starts_with("door");
            collider
                && (object.x - pos.x).abs() < self.tile_width
                && (object.y - pos.y).abs() < self.tile_height
        })
    }

    pub fn in_bounds(&self, tile: Tile) -> bool {
        tile.0 >= 0 && tile.1 >= 0 && tile.0 < self.width && tile.1 < self.height
    }

    pub fn rooms_at(&self, tile: Tile) -> Vec<String> {
        let mut result: Vec<String> = self
            .rooms
            .iter()
            .filter(|(_, set)| set.contains(&tile))
            .map(|(name, _)| name.clone())
            .collect();
        result.sort();
        result
    }

    pub fn visible_tiles(&self, player_tile: Tile) -> HashSet<Tile> {
        let mut set = HashSet::new();
        set.insert(player_tile);
        for room in self.rooms_at(player_tile) {
            if let Some(tiles) = self.rooms.get(&room) {
                set.extend(tiles.iter().copied());
            }
        }
        set
    }

    pub fn draw(&self, tileset: &Texture2D) {
        let tw = self.tile_width;
        let th = self.tile_height;
        for row in 0..self.height as usize {
            for col in 0..self.width as usize {
                let gid = self.tiles[row][col];
                if gid == 0 {
                    continue;
                }
                let local = gid.saturating_sub(1);
                let cols = (tileset.width() / tw) as u32;
                let sx = (local % cols) as f32 * tw;
                let sy = (local / cols) as f32 * th;
                let flags = self.flips[row][col];
                draw_texture_ex(
                    tileset,
                    col as f32 * tw,
                    row as f32 * th,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(sx, sy, tw, th)),
                        dest_size: Some(vec2(tw, th)),
                        rotation: if flags & 4 != 0 { -std::f32::consts::FRAC_PI_2 } else { 0.0 },
                        flip_x: flags & 1 != 0,
                        flip_y: flags & 2 != 0,
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn objects_named_prefix(&self, prefix: &str) -> Vec<&MapObject> {
        let mut result: Vec<&MapObject> = self
            .objects
            .iter()
            .filter(|o| o.name.starts_with(prefix))
            .collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    pub fn object_named(&self, name: &str) -> Option<&MapObject> {
        self.objects.iter().find(|o| o.name == name)
    }

    pub fn object_in_front(&self, pos: Vec2, facing: &str, prefix: &str) -> Option<usize> {
        let (dx, dy) = match facing {
            "up" => (0.0, -1.0),
            "down" => (0.0, 1.0),
            "left" => (-1.0, 0.0),
            _ => (1.0, 0.0),
        };
        let target = pos + vec2(dx, dy) * TILE_SIZE;
        let mut best: Option<(usize, f32)> = None;
        for (index, obj) in self.objects_named_prefix(prefix).iter().enumerate() {
            let d = (vec2(obj.x, obj.y) - target).length();
            if d <= 40.0 && best.map_or(true, |(_, bd)| d < bd) {
                best = Some((index, d));
            }
        }
        best.map(|(index, _)| index)
    }
}

fn collect_layer<'a>(
    layer: &tiled::Layer<'a>,
    width: i32,
    height: i32,
    tiles: &mut [Vec<u32>],
    flips: &mut [Vec<u8>],
    light_sources: &mut Vec<Vec2>,
    solid: &mut HashSet<Tile>,
    spikes: &mut HashSet<Tile>,
    rooms: &mut HashMap<String, HashSet<Tile>>,
    objects: &mut Vec<MapObject>,
    spawn: &mut Vec2,
    ts: f32,
    th: f32,
) {
    match layer.layer_type() {
        LayerType::Group(group) => {
            for child in group.layers() {
                collect_layer(&child, width, height, tiles, flips, light_sources, solid, spikes, rooms, objects, spawn, ts, th);
            }
        }
        LayerType::Tiles(tile_layer) => {
            let name = layer.name.to_lowercase();
            let is_room = name.starts_with("room");
            let mut room_tiles = HashSet::new();
            for row in 0..height {
                for col in 0..width {
                    let Some(tile) = tile_layer.get_tile(col, row) else { continue };
                    let gid = tile.id() + 1;
                    if name == "tiles" {
                        tiles[row as usize][col as usize] = gid;
                        let mut flags = 0u8;
                        if tile.flip_h { flags |= 1; }
                        if tile.flip_v { flags |= 2; }
                        if tile.flip_d { flags |= 4; }
                        flips[row as usize][col as usize] = flags;
                        if gid == LIGHT_SOURCE_TILE_ID {
                            light_sources.push(vec2(col as f32 * ts + ts / 2.0, row as f32 * th + th / 2.0));
                        }
                    }
                    if name == "collision" && gid != 0 {
                        solid.insert(Tile(col, row));
                    }
                    if name == "spikes" && gid != 0 {
                        spikes.insert(Tile(col, row));
                    }
                    if is_room && gid != 0 {
                        room_tiles.insert(Tile(col, row));
                    }
                }
            }
            if is_room {
                rooms.insert(layer.name.clone(), room_tiles);
            }
        }
        LayerType::Objects(object_layer) => {
            for obj in object_layer.objects() {
                let center = vec2(snap_center(obj.x, ts), snap_center(obj.y, ts));
                if obj.name == "playerSpawn" {
                    *spawn = center;
                }
                objects.push(MapObject {
                    x: center.x,
                    y: center.y,
                    name: obj.name.clone(),
                    user_type: obj.user_type.clone(),
                });
            }
        }
        LayerType::Image(_) => {}
    }
}

fn snap_center(value: f32, tile_size: f32) -> f32 {
    ((value - tile_size / 2.0) / tile_size).round() * tile_size + tile_size / 2.0
}

fn object_tile(x: f32, y: f32, tile_width: f32, tile_height: f32) -> Tile {
    Tile((x / tile_width).floor() as i32, (y / tile_height).floor() as i32)
}

pub fn log_line(message: &str) {
    unsafe {
        if let Some(log) = LOG.as_mut() {
            log.log(message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_maps() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
            .join("maps");
        for name in crate::config::MAP_FILES {
            let map = MapData::load(&root.join(name));
            println!(
                "{}: {}x{} solid={} spikes={} objects={} rooms={}",
                map.name,
                map.width,
                map.height,
                map.solid.len(),
                map.spikes.len(),
                map.objects.len(),
                map.rooms.len()
            );
            assert!(map.width > 0 && map.height > 0);
            if name == "map1.tmx" {
                assert!((map.spawn.x - 496.0).abs() < 1.0);
                assert!((map.spawn.y - 80.0).abs() < 1.0);
            }
            if name == "map4.tmx" {
                assert!(map.flips.iter().flatten().any(|flags| *flags & 1 != 0));
            }
        }
    }
}
