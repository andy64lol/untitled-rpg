# Untitled RPG

a smol pixel-art dungeon rpg made with python + arcade, now with a Rust/macroquad port

Explore the dungeon, fight enemies, avoid hazards, and open the chests scattered around each map. Every chest contains a random reward, and the last unopened chest on a floor contains the key to the door. Watch out though, some chests are fakes that only mock you for being so greedy.

## Run it

### Python

```bash
python3 src/main.py
```

### Rust

```bash
cargo run --manifest-path rust-port/Cargo.toml
```

To build or test the Rust port without running it:

```bash
cargo build --manifest-path rust-port/Cargo.toml
cargo test --manifest-path rust-port/Cargo.toml
```

The Rust port keeps its own maps, assets, scripts, saves, and logs under `rust-port/src/`. Saves are separate between the two versions, and the Rust port has no audio yet.

## Goal

Open the chests on map1, take the key they hide, and give it to the door to go down to map2. Do the same for maps 2 through 5 and reach the ending.

## Features

- tile-based dungeon movement
- chest rewards and inventory management
- fake chests that mock your greed
- equipment, combat stats, and enemies that path around walls to reach you
- health, damage, hazards, and healing fountains
- locked doors between maps
- room visibility, light sources, shadows, and fog of war
- simple `.urpgs` script events with nested `run()` calls (`init.urpgs` runs by default on a new game)
- save and load support
- runtime config for the Rust port in `rust-port/.config`

## Controls

| Key | what action they perform |
| --- | --- |
| `WASD` | move |
| `E` / `Enter` | interact |
| `Z` | attack or skip dialogue text |
| `I` | open/close your inventory |
| `Tab` | switch inventory/equipment focus |
| Arrow keys | navigate menus/inventory, or pan the camera with `--freecam` |
| `S` | open Settings from the main menu |
| `1` - `6` | choose a save slot from the main menu or slot screen |
| `Left` / `Right` | cycle save slots from the main menu |
| `Enter` / `Space` | confirm, use an item, or continue |
| `Y` / `N` | confirm or cancel the new-game warning |
| `H` | spawn an enemy at the nearest available `spawnEnemy` marker (debug) |
| `F1` | open/close the in-game controls overlay |
| `M` | minimize/expand the debug panel |
| `E` | open the config editor from the expanded debug panel |
| `Esc` | pause, close a dialogue, return to the main screen |

## Command line flags

Both versions accept the shared flags below (`--help` is available in the Rust port):

| Flag | what it does |
| --- | --- |
| `-h` / `--help` | show help and exit |
| `--lv1` ... `--lv5` | start directly on that map |
| `--health-infinite` | ignore all damage |
| `--freecam` | detach the camera from the player; pan with arrow keys |
| `--disable-player-light` | disable the player's own vision light |
| `--disable-map-light` | disable light sources placed in the map |
| `--disable-rooms` | use light-based visibility instead of room visibility |
| `--disable-fog-of-war` | show all enemies, even on dark tiles |
| `--no-enemies` | disable enemy spawns, movement, and contact damage |
| `--debug` | show modified config values in a panel in the top-right corner |
| `--show-objects` | draw map object markers and labels (Rust only) |
| `--script NAME` | run `scripts/NAME.urpgs` on startup (default: `init`) |
| `--player-moves FILE` | replay `scripts/FILE.urpgs` as player input and start immediately; skips default `init` unless `--script` is also given (Rust only) |
| `--output-file` | also write logs to `logs/runX.txt` |

Example:

```bash
cargo run --manifest-path rust-port/Cargo.toml -- --lv2 --disable-rooms --output-file
```

## Config

The Python version keeps its constants in `src/config.py`.

The Rust version reads simple `key=value` overrides from `rust-port/.config` at startup. It holds window settings, player/combat values, inventory layout, lighting, effects, transition timings, death/win text, and font sizes. Missing or invalid values fall back to built-in defaults. Enemy trigger spawns arm when the player steps on the marker and fire as soon as the player is at least `enemy_spawn_distance_tiles` tiles away (default 2.0); returning to the marker keeps it armed. Enemies spawn at the `spawnEnemy` object's tile-center position and start moving immediately; the spawn log reports that position and tile. Enemies and the player are solid to each other: neither can step onto the other's tile, and enemies attack from an adjacent tile. Enemy `damage` is its attack stat and `defense`/`hp` come from `enemies.json`; contact damage subtracts your equipped defense, while your equipped attack subtracts enemy defense. Hits show a red damage number, flash the enemy red, and start the death animation at 0 HP. Enemy `damage_speed` is a delay in tenths of a second between contact hits, so higher values attack slower and each enemy gets its own `damage_speed / 10` cooldown. Damage logs include the source, enemy spawn and current positions, raw/dealt amount, HP, interval since the previous damage event, and the next cooldown. Spawn markers are numbered numerically (`spawnEnemy2` before `spawnEnemy10`).

The in-game Settings menu is a GUI alternative to the CLI flags. It can change the start map, startup script, infinite health, freecam, lights, room visibility, fog of war, enemies, the debug overlay, log output, and object markers without restarting. Changing a setting reveals the debug panel in the top-right corner.

The debug panel lists only values that differ from the startup config, as `key: value (Original originalValue)`. Press `M` to minimize it to a small `DEBUG` badge without hiding that debug mode is active. Press `E` while expanded to open the config editor: navigate with arrow keys/`W`/`S`, press `Enter` to edit a value, type the new value, and press `Enter` to apply it or `Esc` to cancel. Press `R` in the editor to reset the selected value to its original startup config, or `A` to reset every config value. Runtime config changes are saved in every save slot and restored when that slot is loaded.

## Scripts

Script files use the `.urpgs` extension and live in `src/scripts/` for Python and `rust-port/src/scripts/` for Rust. See `src/scripts/README.txt` for the script syntax. The Rust port also supports `movePlayer()`, `stopMove()`, `face()`, `tick(n)`, `log()`, `interact()`, and `attack()`; `--player-moves FILE` runs a replay script immediately. Enemy spawns are logged with their source, kind, and tile.

## License

The project is released under the MIT License.
