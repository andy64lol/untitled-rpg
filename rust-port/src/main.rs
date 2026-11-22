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
use std::path::PathBuf;

use macroquad::prelude::*;

pub const DEFAULT_STARTUP_SCRIPT: &str = "init";

pub static mut LOG: Option<Logger> = None;

pub struct Logger {
    file: Option<File>,
}

impl Logger {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            file: path.and_then(|p| File::create(p).ok()),
        }
    }

    pub fn set_output(&mut self, enabled: bool) {
        self.file = if enabled {
            next_log_path().and_then(|p| File::create(p).ok())
        } else {
            None
        };
    }

    pub fn log(&mut self, message: &str) {
        println!("{message}");
        if let Some(file) = self.file.as_mut() {
            let _ = writeln!(file, "{message}");
            let _ = file.flush();
        }
    }

    pub fn write_only(&mut self, message: &str) {
        if let Some(file) = self.file.as_mut() {
            let _ = writeln!(file, "{message}");
            let _ = file.flush();
        }
    }
}

fn next_log_path() -> Option<PathBuf> {
    let dir = assets::find_project_root().join("src").join("logs");
    std::fs::create_dir_all(&dir).ok()?;
    let mut number = 1;
    while dir.join(format!("run{number}.txt")).exists() {
        number += 1;
    }
    Some(dir.join(format!("run{number}.txt")))
}

pub fn set_log_output(enabled: bool) {
    unsafe {
        if let Some(log) = LOG.as_mut() {
            log.set_output(enabled);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cli {
    pub level: usize,
    pub level_given: bool,
    pub health_infinite: bool,
    pub disable_player_light: bool,
    pub freecam: bool,
    pub disable_map_light: bool,
    pub disable_rooms: bool,
    pub disable_fog_of_war: bool,
    pub no_enemies: bool,
    pub debug: bool,
    pub script: Option<String>,
    pub output_file: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            level: 0,
            level_given: false,
            health_infinite: false,
            disable_player_light: false,
            freecam: false,
            disable_map_light: false,
            disable_rooms: false,
            disable_fog_of_war: false,
            no_enemies: false,
            debug: false,
            script: Some(DEFAULT_STARTUP_SCRIPT.into()),
            output_file: false,
        }
    }
}

fn print_help() {
    println!("Untitled RPG");
    println!();
    println!("Usage: untitled-rpg [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -h, --help            show this help and exit");
    for level in 1..=config::MAP_FILES.len() {
        println!("  --lv{level:<10}       start directly on map {level}");
    }
    println!("  --health-infinite     ignore all damage");
    println!("  --freecam             detach the camera; pan with arrow keys");
    println!("  --disable-player-light    disable the player's vision light");
    println!("  --disable-map-light       disable light sources placed in the map");
    println!("  --disable-rooms           use light visibility instead of rooms");
    println!("  --disable-fog-of-war      show enemies even on dark tiles");
    println!("  --no-enemies              disable enemy spawns and contact damage");
    println!("  --debug                   show modified config values in the top-right corner");
    println!("  --script NAME             run scripts/NAME.urpgs on startup (default: init)");
    println!("  --output-file             also write logs to src/logs/runX.txt");
    println!();
    println!("The same options can be changed in-game from the Settings menu.");
}

fn parse_args() -> Result<Option<Cli>, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    parse_args_from(&args)
}

fn parse_args_from(args: &[String]) -> Result<Option<Cli>, String> {
    let mut cli = Cli::default();
    let mut level = None;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        if arg == "-h" || arg == "--help" {
            return Ok(None);
        }
        if let Some(value) = arg.strip_prefix("--lv") {
            let number = value.parse::<usize>().map_err(|_| format!("invalid level flag: {arg}"))?;
            if number == 0 || number > config::MAP_FILES.len() {
                return Err(format!("level must be between 1 and {}", config::MAP_FILES.len()));
            }
            if level.replace(number - 1).is_some() {
                return Err("only one --lv flag can be used at a time".into());
            }
            index += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--script=") {
            if value.is_empty() {
                return Err("--script requires a script name".into());
            }
            cli.script = Some(value.to_string());
            index += 1;
            continue;
        }
        match arg.as_str() {
            "--health-infinite" => cli.health_infinite = true,
            "--freecam" => cli.freecam = true,
            "--disable-player-light" => cli.disable_player_light = true,
            "--disable-map-light" => cli.disable_map_light = true,
            "--disable-rooms" => cli.disable_rooms = true,
            "--disable-fog-of-war" => cli.disable_fog_of_war = true,
            "--no-enemies" => cli.no_enemies = true,
            "--debug" => cli.debug = true,
            "--output-file" => cli.output_file = true,
            "--script" => {
                let value = args.get(index + 1).ok_or("--script requires a script name")?;
                if value.starts_with('-') {
                    return Err("--script requires a script name, not another flag".into());
                }
                cli.script = Some(value.clone());
                index += 1;
            }
            _ => return Err(format!("unknown option: {arg}")),
        }
        index += 1;
    }

    if let Some(level) = level {
        cli.level = level;
        cli.level_given = true;
    }
    Ok(Some(cli))
}

fn window_conf() -> Conf {
    config::init(&assets::find_project_root().join(".config"));
    match parse_args() {
        Ok(Some(_)) => {}
        Ok(None) => {
            print_help();
            std::process::exit(0);
        }
        Err(message) => {
            eprintln!("error: {message}\n");
            print_help();
            std::process::exit(2);
        }
    }
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
    let cli = match parse_args() {
        Ok(Some(cli)) => cli,
        Ok(None) => {
            print_help();
            return;
        }
        Err(message) => {
            eprintln!("error: {message}");
            return;
        }
    };
    let log_path = if cli.output_file { next_log_path() } else { None };
    unsafe {
        LOG = Some(Logger::new(log_path.clone()));
    }
    if let Some(path) = log_path {
        let log_name = path.file_name().and_then(|value| value.to_str()).unwrap_or("run.txt");
        let flags: Vec<String> = std::env::args().skip(1).collect();
        let flags = if flags.is_empty() { "(none)".into() } else { flags.join(" ") };
        let header = format!(
            "============================================================\nlog:   {log_name}\nflags: {flags}\n============================================================"
        );
        unsafe {
            if let Some(log) = LOG.as_mut() {
                log.write_only(&header);
            }
        }
    }

    let assets = assets::Assets::load().await;
    let mut game = game::Game::new(assets, cli);
    game.run().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn startup_script_defaults_to_init() {
        let cli = parse_args_from(&[]).unwrap().unwrap();
        assert_eq!(cli.script.as_deref(), Some("init"));
    }

    #[test]
    fn help_flags_exit_cleanly() {
        assert!(parse_args_from(&args(&["--help"])).unwrap().is_none());
        assert!(parse_args_from(&args(&["-h"])).unwrap().is_none());
    }

    #[test]
    fn parses_all_feature_flags() {
        let cli = parse_args_from(&args(&[
            "--lv3",
            "--health-infinite",
            "--freecam",
            "--disable-player-light",
            "--disable-map-light",
            "--disable-rooms",
            "--disable-fog-of-war",
            "--no-enemies",
            "--debug",
            "--script",
            "example",
            "--output-file",
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(cli.level, 2);
        assert!(cli.level_given);
        assert!(cli.health_infinite);
        assert!(cli.freecam);
        assert!(cli.disable_player_light);
        assert!(cli.disable_map_light);
        assert!(cli.disable_rooms);
        assert!(cli.disable_fog_of_war);
        assert!(cli.no_enemies);
        assert!(cli.debug);
        assert_eq!(cli.script.as_deref(), Some("example"));
        assert!(cli.output_file);
    }

    #[test]
    fn parses_equals_script_form() {
        let cli = parse_args_from(&args(&["--script=example"])).unwrap().unwrap();
        assert_eq!(cli.script.as_deref(), Some("example"));
    }

    #[test]
    fn rejects_invalid_flags() {
        assert!(parse_args_from(&args(&["--lv0"])).is_err());
        assert!(parse_args_from(&args(&["--lv99"])).is_err());
        assert!(parse_args_from(&args(&["--lv1", "--lv2"])).is_err());
        assert!(parse_args_from(&args(&["--script"])).is_err());
        assert!(parse_args_from(&args(&["--script", "--freecam"])).is_err());
        assert!(parse_args_from(&args(&["--bogus"])).is_err());
    }
}
