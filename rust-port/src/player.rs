use macroquad::prelude::*;

use crate::assets::Assets;
use crate::config::*;
use crate::map::{MapData, Tile};

pub struct Player {
    pub pos: Vec2,
    pub target: Vec2,
    pub moving: bool,
    pub just_finished_move: bool,
    pub facing: String,
    pub direction: String,
    pub hp: i32,
    pub max_hp: i32,
    pub dead: bool,
    pub invincible: bool,
    pub angle: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub alpha: f32,
    pub color: Color,
    pub swing_phase: f32,
    pub flip_timer: f32,
    pub damage_flash_timer: f32,
    pub death_anim_time: f32,
}

impl Player {
    pub fn new(spawn: Vec2, invincible: bool) -> Self {
        Self {
            pos: spawn,
            target: spawn,
            moving: false,
            just_finished_move: false,
            facing: "right".into(),
            direction: "right".into(),
            hp: crate::config::get().player_max_hp,
            max_hp: crate::config::get().player_max_hp,
            dead: false,
            invincible,
            angle: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            alpha: 1.0,
            color: WHITE,
            swing_phase: 0.0,
            flip_timer: 0.0,
            damage_flash_timer: 0.0,
            death_anim_time: 0.0,
        }
    }

    pub fn reset(&mut self, spawn: Vec2) {
        self.pos = spawn;
        self.target = spawn;
        self.moving = false;
        self.just_finished_move = false;
        self.facing = "right".into();
        self.direction = "right".into();
        self.hp = self.max_hp;
        self.dead = false;
        self.angle = 0.0;
        self.scale_x = 1.0;
        self.scale_y = 1.0;
        self.alpha = 1.0;
        self.color = WHITE;
        self.swing_phase = 0.0;
        self.flip_timer = 0.0;
        self.damage_flash_timer = 0.0;
        self.death_anim_time = 0.0;
    }

    pub fn set_facing(&mut self, facing: &str) {
        if self.dead {
            return;
        }
        self.facing = facing.to_string();
        if facing == "left" || facing == "right" {
            if facing != self.direction {
                self.flip_timer = crate::config::get().flip_duration;
                self.swing_phase = 0.0;
            }
            self.direction = facing.to_string();
        }
    }

    pub fn facing_offset(&self) -> Vec2 {
        match self.facing.as_str() {
            "up" => vec2(0.0, -1.0),
            "down" => vec2(0.0, 1.0),
            "left" => vec2(-1.0, 0.0),
            _ => vec2(1.0, 0.0),
        }
    }

    pub fn attack_position(&self) -> Vec2 {
        self.pos + self.facing_offset() * TILE_SIZE
    }

    pub fn take_damage(&mut self, damage: i32, defense: i32) -> i32 {
        if self.dead || self.invincible {
            return 0;
        }
        let dealt = (damage - defense).max(0);
        self.hp -= dealt;
        if dealt > 0 {
            self.damage_flash_timer = crate::config::get().damage_flash_duration;
        }
        if self.hp <= 0 {
            self.hp = 0;
            self.dead = true;
            self.moving = false;
            self.death_anim_time = 0.0;
            self.angle = 0.0;
            self.flip_timer = 0.0;
            self.scale_x = 1.0;
            self.scale_y = 1.0;
            self.alpha = 1.0;
        }
        dealt
    }

    pub fn heal(&mut self, amount: i32) {
        if self.dead {
            return;
        }
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn update_death_animation(&mut self, dt: f32) {
        if !self.dead {
            return;
        }
        self.death_anim_time += dt;
        let progress = (self.death_anim_time / crate::config::get().death_animation_duration).min(1.0);
        self.scale_x = 1.0 + 0.12 * (std::f32::consts::PI * progress).sin();
        self.scale_y = 1.0 - 0.32 * progress;
        self.alpha = 1.0 - 0.35 * progress;
        self.angle = 0.0;
    }

    fn update_animation(&mut self, dt: f32) {
        if self.dead {
            self.color = WHITE;
            return;
        }
        let mut swing_target = 0.0;
        if self.moving {
            let sign = if self.direction == "left" { -1.0 } else { 1.0 };
            swing_target = crate::config::get().swing_angle_degrees * sign
                * (0.75 + 0.25 * self.swing_phase.sin());
            self.swing_phase += dt * crate::config::get().swing_speed;
        }
        let ease = (dt * crate::config::get().swing_ease).min(1.0);
        self.angle += (swing_target - self.angle) * ease;
        if self.angle.abs() < 0.01 {
            self.angle = 0.0;
        }
        if self.flip_timer > 0.0 {
            self.flip_timer = (self.flip_timer - dt).max(0.0);
            let progress = 1.0 - self.flip_timer / crate::config::get().flip_duration;
            self.scale_x = (std::f32::consts::PI * progress).cos().abs();
            if self.flip_timer == 0.0 {
                self.scale_x = 1.0;
            }
        } else {
            self.scale_x = 1.0;
        }
        if self.damage_flash_timer > 0.0 {
            self.damage_flash_timer = (self.damage_flash_timer - dt).max(0.0);
            let progress = self.damage_flash_timer / crate::config::get().damage_flash_duration;
            self.color = Color::new(1.0, 110.0 / 255.0, 110.0 / 255.0, 1.0);
            self.scale_y = 1.0 - 0.12 * progress;
        } else {
            self.color = WHITE;
            self.scale_y = 1.0;
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        map: &MapData,
        blocked: &std::collections::HashSet<Tile>,
        keys: &[KeyCode],
        pressed: &[KeyCode],
    ) -> bool {
        self.just_finished_move = false;
        self.update_animation(dt);
        if self.dead {
            self.update_death_animation(dt);
            return false;
        }

        if self.moving {
            let delta = self.target - self.pos;
            let distance = delta.length();
            let step = crate::config::get().player_speed * dt;
            if distance <= step || distance < 0.01 {
                self.pos = self.target;
                self.moving = false;
                self.just_finished_move = true;
                return true;
            } else {
                self.pos += delta.normalize() * step;
            }
            return false;
        }

        for key in pressed {
            match key {
                KeyCode::W => self.set_facing("up"),
                KeyCode::S => self.set_facing("down"),
                KeyCode::A => self.set_facing("left"),
                KeyCode::D => self.set_facing("right"),
                _ => {}
            }
        }

        let (dx, dy) = if keys.contains(&KeyCode::W) {
            (0.0, -1.0)
        } else if keys.contains(&KeyCode::S) {
            (0.0, 1.0)
        } else if keys.contains(&KeyCode::A) {
            (-1.0, 0.0)
        } else if keys.contains(&KeyCode::D) {
            (1.0, 0.0)
        } else {
            (0.0, 0.0)
        };
        if dx == 0.0 && dy == 0.0 {
            return false;
        }
        let facing = if dy < 0.0 {
            "up"
        } else if dy > 0.0 {
            "down"
        } else if dx < 0.0 {
            "left"
        } else {
            "right"
        };
        self.set_facing(facing);
        let from = map.tile_of(self.pos);
        let next = Tile(from.0 + dx as i32, from.1 + dy as i32);
        if !map.in_bounds(next) || map.is_solid(next) || blocked.contains(&next) {
            return false;
        }
        self.target = MapData::center_of(next);
        self.moving = true;
        false
    }

    pub fn draw(&self, assets: &Assets) {
        let frame = if self.direction == "left" {
            if self.dead { 1 } else { 0 }
        } else if self.dead {
            3
        } else {
            2
        };
        let sx = (frame % 8) as f32 * TILE_SIZE;
        let sy = (frame / 8) as f32 * TILE_SIZE;
        let w = TILE_SIZE * self.scale_x.abs();
        let h = TILE_SIZE * self.scale_y;
        let alpha = (self.alpha * 255.0) as u8;
        draw_texture_ex(
            &assets.player,
            self.pos.x - w / 2.0,
            self.pos.y - h / 2.0,
            Color::new(self.color.r, self.color.g, self.color.b, alpha as f32 / 255.0),
            DrawTextureParams {
                source: Some(Rect::new(sx, sy, TILE_SIZE, TILE_SIZE)),
                dest_size: Some(vec2(w, h)),
                rotation: -self.angle.to_radians(),
                flip_x: self.scale_x < 0.0,
                ..Default::default()
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::MapData;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn reports_finished_tile_move() {
        let map = MapData {
            name: "test".into(),
            width: 4,
            height: 4,
            tile_width: 32.0,
            tile_height: 32.0,
            tiles: vec![vec![0; 4]; 4],
            flips: vec![vec![0; 4]; 4],
            solid: HashSet::new(),
            spikes: HashSet::new(),
            objects: Vec::new(),
            rooms: HashMap::new(),
            spawn: vec2(48.0, 48.0),
            light_sources: Vec::new(),
            light_blockers: HashSet::new(),
            lightable_tiles: HashSet::new(),
            block_light_tiles: HashSet::new(),
        };
        let mut player = Player::new(vec2(48.0, 48.0), false);
        player.moving = true;
        player.target = vec2(80.0, 48.0);
        let finished = player.update(1.0, &map, &HashSet::new(), &[], &[]);
        assert!(finished);
        assert!(player.just_finished_move);
        assert_eq!(player.pos.x, 80.0);
    }

    #[test]
    fn taking_damage_shows_red_flash() {
        let mut player = Player::new(vec2(48.0, 48.0), false);
        assert_eq!(player.take_damage(10, 1), 9);
        assert!(player.damage_flash_timer > 0.0);

        player.update_animation(0.01);
        assert_eq!(player.color.r, 1.0);
        assert_eq!(player.color.g, 110.0 / 255.0);
        assert_eq!(player.color.b, 110.0 / 255.0);
    }

    #[test]
    fn cannot_start_move_onto_blocked_tile() {
        let map = MapData {
            name: "test".into(),
            width: 4,
            height: 4,
            tile_width: 32.0,
            tile_height: 32.0,
            tiles: vec![vec![0; 4]; 4],
            flips: vec![vec![0; 4]; 4],
            solid: HashSet::new(),
            spikes: HashSet::new(),
            objects: Vec::new(),
            rooms: HashMap::new(),
            spawn: vec2(48.0, 48.0),
            light_sources: Vec::new(),
            light_blockers: HashSet::new(),
            lightable_tiles: HashSet::new(),
            block_light_tiles: HashSet::new(),
        };
        let mut blocked = HashSet::new();
        blocked.insert(Tile(2, 1));
        let mut player = Player::new(vec2(48.0, 48.0), false);
        assert!(!player.update(0.016, &map, &blocked, &[KeyCode::D], &[]));
        assert!(!player.moving);
        assert_eq!(player.target, vec2(48.0, 48.0));
    }
}
