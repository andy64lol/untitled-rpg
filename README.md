# Untitled RPG

a smol pixel-art dungeon rpg made with python and arcade

Explore the dungeon, fight your way through hazards (no monsters added rn), and open the chests scattered around the map. each chest contains a random reward, and the last unopened chest on a floor contains the key to the door. watch out though, some chests are fakes that only mock you for being so greedy

just run main.py from src with 

```bash
python3 src/main.py
```

## Goal

Open the chests on map1, take the key they hide, and give it to the door to go down to map2. do the same on map2 and repeat

## Features

- tile-based dungeon movement
- chest rewards and inventory management
- fake chests that mock your greed
- equipment and combat stats
- health, damage, hazards, and healing fountains
- locked doors that take you from map1 to map2, then to the ending
- save and load support

## Controls

| Key | what action they perform |
| --- | --- |
| `WASD` | move |
| `E` / `Enter` | Interact |
| `Z` | attack or damage |
| `I` | Open your inventory |
| `H` | Take damage (debug), don't use in game |
| `Esc` | Pause or close a dialogue, main screen if in none |
| `Space` | Confirm dialogue choices |

The project is released under the MIT License.
