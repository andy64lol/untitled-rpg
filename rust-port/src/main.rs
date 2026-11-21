#![allow(static_mut_refs)]

mod assets;
mod config;
mod enemies;
mod game;
mod items;
mod map;
mod player;
mod save;
mod script;

use std::fs::File;
use std::io::Write;

use macroquad::prelude::*;

pub static mut LOG: Option<Logger> = None;

pub struct Logger {
    file: Option<File>,
}

impl Logger {
    pub fn new(path: Option<std::path::PathBuf>) -> Self {
        let file = path.and_then(|p| File::create(p).ok());
        Self { file }
    }

    pub fn log(&mut self, message: &str) {
        println!("{message}");
        if let Some(file) = self.file.as_mut() {
            let _ = writeln!(file, "{message}");
            let _ = file.flush();
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Cli {
    pub level: usize,
    pub level_given: bool,
    pub health_infinite: bool,
    pub disable_player_light: bool,
    pub freecam: bool,
    pub disable_map_light: bool,
    pub disable_rooms: bool,
    pub disable_fog_of_war: bool,
    pub script: Option<String>,
    pub output_file: bool,
}

fn parse_args() -> Cli {
    let mut cli = Cli::default();
    let args: Vec<String> = std::env::args().skip(1).collect();
    for arg in &args {
        if let Some(index) = arg.strip_prefix("--lv") {
            if let Ok(level) = index.parse::<usize>() {
                if level >= 1 && level <= config::MAP_FILES.len() {
                    cli.level = level - 1;
                    cli.level_given = true;
                }
            }
        }
        match arg.as_str() {
            "--health-infinite" => cli.health_infinite = true,
            "--disable-player-light" => cli.disable_player_light = true,
            "--freecam" => cli.freecam = true,
            "--disable-map-light" => cli.disable_map_light = true,
            "--disable-rooms" => cli.disable_rooms = true,
            "--disable-fog-of-war" => cli.disable_fog_of_war = true,
            "--output-file" => cli.output_file = true,
            _ => {}
        }
    }
    if let Some(index) = args.iter().position(|a| a == "--script") {
        cli.script = args.get(index + 1).cloned();
    }
    cli
}

fn window_conf() -> Conf {
    config::init(&assets::find_project_root().join(".config"));
    let settings = config::get();
    Conf {
        window_title: settings.window_title.clone(),
        window_width: settings.window_width,
        window_height: settings.window_height,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let cli = parse_args();
    let log_path = if cli.output_file {
        let dir = assets::find_project_root().join("src").join("logs");
        let _ = std::fs::create_dir_all(&dir);
        let mut number = 1;
        while dir.join(format!("run{number}.txt")).exists() {
            number += 1;
        }
        Some(dir.join(format!("run{number}.txt")))
    } else {
        None
    };
    unsafe {
        LOG = Some(Logger::new(log_path));
    }

    let assets = assets::Assets::load().await;
    let mut game = game::Game::new(assets, cli);
    game.run().await;
}
