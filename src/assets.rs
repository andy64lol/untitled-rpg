use std::collections::HashMap;
use std::path::{Path, PathBuf};

use macroquad::prelude::*;
use macroquad::text::load_ttf_font;

pub fn find_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub struct Assets {
    pub root: PathBuf,
    pub font: Font,
    pub tiles: Texture2D,
    pub player: Texture2D,
    pub heart: Texture2D,
    pub broken_heart: Texture2D,
    pub dialogue_box: Texture2D,
    pub title: Texture2D,
    pub item_textures: HashMap<String, Texture2D>,
    pub mob_textures: HashMap<String, Texture2D>,
    pub slot_textures: HashMap<String, Texture2D>,
    pub buttons: HashMap<String, Texture2D>,
}

impl Assets {
    pub async fn load() -> Self {
        let root = find_project_root();
        let src = root.join("src");
        let font = load_ttf_font(&path_str(&src.join("assets/UI/font/PixeloidMono-nAOpP.ttf")))
            .await
            .expect("font");
        let tiles = load_nearest(&path_str(&src.join("assets/tiles/dungeon1.png"))).await;
        let player = load_nearest(&path_str(&src.join("assets/sprites/player.png"))).await;
        let heart = load_nearest(&path_str(&src.join("assets/UI/heart.png"))).await;
        let broken_heart = load_nearest(&path_str(&src.join("assets/UI/broken_heart.png"))).await;
        let dialogue_box = load_nearest(&path_str(&src.join("assets/UI/dialogue/dialogue_box.png"))).await;
        let title = load_nearest(&path_str(&src.join("assets/UI/menu/game_title.png"))).await;

        let mut item_textures = HashMap::new();
        if let Ok(dir) = std::fs::read_dir(src.join("assets/items")) {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("png") {
                    let name = path.file_name().unwrap().to_string_lossy().to_string();
                    if let Ok(texture) = load_nearest_opt(&path_str(&path)).await {
                        item_textures.insert(name, texture);
                    }
                }
            }
        }

        let mut mob_textures = HashMap::new();
        let mob_dir = src.join("assets/sprites/enemy");
        if let Ok(dir) = std::fs::read_dir(&mob_dir) {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("png") {
                    let name = path.file_name().unwrap().to_string_lossy().to_string();
                    if let Ok(texture) = load_nearest_opt(&path_str(&path)).await {
                        mob_textures.insert(name, texture);
                    }
                }
            }
        }

        let mut slot_textures = HashMap::new();
        if let Ok(dir) = std::fs::read_dir(src.join("assets/UI/player_equipment_slots")) {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("png") {
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    if let Ok(texture) = load_nearest_opt(&path_str(&path)).await {
                        slot_textures.insert(name, texture);
                    }
                }
            }
        }

        let mut buttons = HashMap::new();
        if let Ok(dir) = std::fs::read_dir(src.join("assets/UI/buttons")) {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("png") {
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    if let Ok(texture) = load_nearest_opt(&path_str(&path)).await {
                        buttons.insert(name, texture);
                    }
                }
            }
        }

        Self {
            root,
            font,
            tiles,
            player,
            heart,
            broken_heart,
            dialogue_box,
            title,
            item_textures,
            mob_textures,
            slot_textures,
            buttons,
        }
    }

    pub fn src(&self) -> PathBuf {
        self.root.join("src")
    }

    pub fn item_texture(&self, name: &str) -> Option<Texture2D> {
        self.item_textures.get(name).cloned()
    }
}

async fn load_nearest(path: &str) -> Texture2D {
    let texture = load_texture(path).await.expect("texture");
    texture.set_filter(FilterMode::Nearest);
    texture
}

async fn load_nearest_opt(path: &str) -> Result<Texture2D, macroquad::Error> {
    let texture = load_texture(path).await?;
    texture.set_filter(FilterMode::Nearest);
    Ok(texture)
}

pub fn path_str(path: &Path) -> String {
    path.to_string_lossy().to_string()
}


pub fn wrap_text(text: &str, size: u16, font: &Font, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if !line.is_empty() && measure_text(&candidate, Some(font), size, 1.0).width > max_width {
                lines.push(line);
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        lines.push(line);
    }
    lines
}

pub fn draw_wrapped_text(
    text: &str,
    x: f32,
    y: f32,
    size: u16,
    color: Color,
    font: &Font,
    max_width: f32,
    line_height: f32,
) {
    for (index, line) in wrap_text(text, size, font, max_width).iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        draw_text_at(
            line,
            x,
            y + index as f32 * line_height,
            size,
            color,
            font,
            0.0,
            0.0,
        );
    }
}

pub fn draw_text_at(
    text: &str,
    x: f32,
    y: f32,
    size: u16,
    color: Color,
    font: &Font,
    anchor_x: f32,
    anchor_y: f32,
) {
    let dims = measure_text(text, Some(font), size, 1.0);
    let px = x - dims.width * anchor_x;
    let py = y - dims.height * anchor_y + dims.offset_y;
    let params = TextParams {
        font: Some(font),
        font_size: size,
        color,
        ..Default::default()
    };
    let blur = Color::new(color.r, color.g, color.b, color.a * 0.12);
    for (offset_x, offset_y) in [(-0.6, 0.0), (0.6, 0.0), (0.0, -0.6), (0.0, 0.6)] {
        draw_text_ex(
            text,
            px + offset_x,
            py + offset_y,
            TextParams { color: blur, ..params },
        );
    }
    draw_text_ex(text, px, py, params);
}
