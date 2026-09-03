import math
from dataclasses import dataclass
from typing import Any

import arcade
from camera import CameraController
from chests import ChestManager
from config import *
from dialogue import DialogueBox, OptionBox
from enemy import EnemyManager
from inventory import ConfirmResult, Equipment, Inventory, InventoryScreen
from items import get_combat_stats
from load import load_state
from map import GameMap
from menu import MainMenu
import save
from sprites import Player


@dataclass
class AttackEffect:
    center_x: float
    center_y: float
    timer: float = ATTACK_EFFECT_DURATION
    size: float = ATTACK_EFFECT_SIZE

    def update(self, delta_time: float) -> bool:
        self.timer -= delta_time
        return self.timer > 0

    def draw(self) -> None:
        half = self.size / 2
        alpha = max(
            0,
            min(255, int(255 * (self.timer / ATTACK_EFFECT_DURATION))),
        )
        arcade.draw_lrbt_rectangle_filled(
            self.center_x - half,
            self.center_x + half,
            self.center_y - half,
            self.center_y + half,
            (255, 220, 120, alpha),
        )


class GameWindow(arcade.Window):
    def __init__(self):
        super().__init__(
            WINDOW_WIDTH, WINDOW_HEIGHT, WINDOW_TITLE, resizable=True
        )
        self.set_minimum_size(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT)
        arcade.set_background_color(WINDOW_BACKGROUND)

        self.camera = CameraController(self.width, self.height)
        self.menu = MainMenu(self.width, self.height, ASSETS_DIR)
        self.currentScreen = "menu"
        self.keys: set[int] = set()
        self.stat_top = 0
        self.stat_bottom = 0

        self.map_index = 0
        self.game_map = GameMap(MAPS_DIR / MAP_FILES[self.map_index])
        self.chests = ChestManager(
            self.game_map.chests,
            self.game_map.fake_chests,
            self.game_map.tilemap.tile_width,
            self.game_map.tilemap.tile_height,
        )
        self.enemies = EnemyManager(
            self.game_map.collision,
            self.game_map.enemy_spawns,
            self.game_map.tilemap.tile_width,
            self.game_map.tilemap.tile_height,
        )
        self.camera.set_world_bounds(
            0,
            self.game_map.world_width,
            0,
            self.game_map.world_height,
        )
        self.player = Player(
            self.game_map.tilemap,
            self.game_map.collision,
            self.game_map.fountain_colliders,
        )
        self.player_list = arcade.SpriteList()
        self.player_list.append(self.player)

        self.heart_texture = arcade.load_texture(str(HEART_PATH))
        self.broken_heart_texture = arcade.load_texture(str(BROKEN_HEART_PATH))
        self.heart_list = arcade.SpriteList()
        self._create_health_hud()
        self._create_combat_hud()

        self.dialogue = DialogueBox(self.width, self.height)
        self.options = OptionBox(self.width, self.height)
        self.fountain_cooldown = 0.0
        self.pending_choice = ""
        self.won = False
        self.transition_phase = ""
        self.transition_elapsed = 0.0
        self.transition_target_map: int | None = None

        self.inventory = Inventory()
        self.equipment = Equipment()
        self.inventory_screen = InventoryScreen(
            self.width, self.height, self.inventory, self.equipment
        )
        self.gold = STARTING_GOLD
        self.inventory_screen.set_gold(self.gold)
        self._seed_inventory()

        self.attack_effects: list[AttackEffect] = []
        self.heart_shake_time = 0
        self.heart_shake_strength = 2
        self.spike_damage_cooldown = 1.0
        self.spike_damage_timer = 0
        self.enemy_damage_cooldown = 0.75
        self.enemy_damage_timer = 0.0

    def on_resize(self, width: int, height: int) -> None:
        super().on_resize(width, height)
        self.camera.resize(width, height)
        self.menu.resize(width, height)
        self.dialogue.resize(width, height)
        self.options.resize(width, height)
        self.inventory_screen.resize(width, height)
        self._layout_health_hud(height)
        self._layout_combat_hud()

        if self.currentScreen == "game":
            self.camera.follow(self.player.center_x, self.player.center_y)

    def _create_health_hud(self) -> None:
        heart_count = self.player.maxHp // HEALTH_PER_HEART

        for i in range(heart_count):
            heart = arcade.Sprite(
                self.heart_texture, scale=HEART_SCALE, pixelated=True
            )
            heart.width = HEART_SIZE
            heart.height = HEART_SIZE
            self.heart_list.append(heart)

        self._layout_health_hud(self.height)

    def _layout_health_hud(self, height: int | None = None) -> None:
        hud_height = height if height is not None else self.height
        for index, heart in enumerate(self.heart_list):
            heart.center_x = self._heart_x(index)
            heart.center_y = hud_height - HUD_TOP_OFFSET

    def _heart_x(self, index: int) -> float:
        return HUD_LEFT + index * (HEART_SIZE + HEART_GAP)

    def _create_combat_hud(self) -> None:
        self.atk_box_left = ATTACK_BOX_LEFT
        self.def_box_left = DEFENSE_BOX_LEFT

        self.atk_value = arcade.Text(
            "1",
            0,
            0,
            STAT_TEXT_COLOR,
            font_size=STAT_VALUE_FONT_SIZE,
            font_name=FONT_NAME,
            anchor_x="center",
            anchor_y="center",
        )
        self.def_value = arcade.Text(
            "1",
            0,
            0,
            STAT_TEXT_COLOR,
            font_size=STAT_VALUE_FONT_SIZE,
            font_name=FONT_NAME,
            anchor_x="center",
            anchor_y="center",
        )
        self._layout_combat_hud()

    def _layout_combat_hud(self) -> None:
        self.stat_top = self.height - STAT_TOP_OFFSET
        self.stat_bottom = self.height - STAT_BOTTOM_OFFSET
        center_y = (self.stat_top + self.stat_bottom) / 2

        self.atk_value.x = self.atk_box_left + STAT_VALUE_OFFSET
        self.atk_value.y = center_y
        self.def_value.x = self.def_box_left + STAT_VALUE_OFFSET
        self.def_value.y = center_y

    @property
    def map_name(self) -> str:
        return MAP_FILES[self.map_index]

    @property
    def is_last_map(self) -> bool:
        return self.map_index >= len(MAP_FILES) - 1

    @classmethod
    def map_index_of(cls, map_name: Any) -> int:
        if isinstance(map_name, str) and map_name in MAP_FILES:
            return MAP_FILES.index(map_name)

        return 0

    def load_map(self, index: int) -> None:
        self.map_index = max(0, min(index, len(MAP_FILES) - 1))
        self.game_map = GameMap(MAPS_DIR / self.map_name)
        self.chests = ChestManager(
            self.game_map.chests,
            self.game_map.fake_chests,
            self.game_map.tilemap.tile_width,
            self.game_map.tilemap.tile_height,
        )
        self.enemies = EnemyManager(
            self.game_map.collision,
            self.game_map.enemy_spawns,
            self.game_map.tilemap.tile_width,
            self.game_map.tilemap.tile_height,
        )
        self.camera.set_world_bounds(
            0,
            self.game_map.world_width,
            0,
            self.game_map.world_height,
        )
        self.player.set_map(
            self.game_map.collision,
            self.game_map.fountain_colliders,
            self.game_map.spawn[0],
            self.game_map.spawn[1],
        )
        self.fountain_cooldown = 0.0
        self.enemy_damage_timer = 0.0
        self.camera.follow(self.player.center_x, self.player.center_y)

    def start_game(self) -> None:
        self.load_map(0)
        self.player.reset()
        self.won = False
        self.gold = STARTING_GOLD
        self._seed_inventory()
        self.update_health()
        self.update_combat_hud()
        self._enter_game()

    def enter_next_map(self) -> None:
        self.load_map(self.map_index + 1)
        self.dialogue.show(NEXT_MAP_MESSAGE)

    @property
    def transition_active(self) -> bool:
        return bool(self.transition_phase)

    def begin_map_transition(self) -> None:
        if self.is_last_map or self.transition_active:
            return

        self.transition_phase = "out"
        self.transition_elapsed = 0.0
        self.transition_target_map = self.map_index + 1
        self.keys.clear()

    def update_transition(self, delta_time: float) -> None:
        self.transition_elapsed += delta_time
        if self.transition_elapsed < TRANSITION_DURATION:
            return

        if self.transition_phase == "out":
            target_map = self.transition_target_map
            if target_map is not None:
                self.load_map(target_map)
                self.dialogue.show(NEXT_MAP_MESSAGE)
            self.transition_phase = "in"
            self.transition_elapsed = 0.0
            self.transition_target_map = None
            return

        self.transition_phase = ""
        self.transition_elapsed = 0.0

    def draw_transition(self) -> None:
        if not self.transition_active:
            return

        progress = min(
            self.transition_elapsed / TRANSITION_DURATION,
            1.0,
        )
        alpha = progress if self.transition_phase == "out" else 1.0 - progress
        self.camera.use_gui()
        arcade.draw_lrbt_rectangle_filled(
            0,
            self.width,
            0,
            self.height,
            (0, 0, 0, int(alpha * 255)),
        )

    def resume_game(self) -> None:
        self._enter_game()

    def pause_game(self) -> None:
        self.currentScreen = "menu"
        self.menu.can_resume = True
        self.menu.status = ""
        self.keys.clear()

    def save_game(self) -> None:
        save.save_state(SAVE_PATH, self._build_save_state())
        self.menu.status = "Game saved."
        self.menu.can_resume = True

    def load_game(self) -> None:
        state = load_state(SAVE_PATH)
        if state is None:
            self.menu.status = "No saved game found."
            return

        self._apply_save_state(state)
        self.update_health()
        self.camera.follow(self.player.center_x, self.player.center_y)
        if self.won:
            self.currentScreen = "won"
            self.menu.can_resume = False
            self.keys.clear()
        else:
            self._enter_game()

    def _build_save_state(self) -> dict[str, Any]:
        return {
            "map": self.map_name,
            "player": self.player.saveState(),
            "inventory": self.inventory.save_state(),
            "equipment": self.equipment.save_state(),
            "gold": self.gold,
            "chests": self.chests.save_state(),
            "enemies": self.enemies.save_state(),
            "map_objects": self.game_map.save_state(),
            "door_unlocked": self.won,
        }

    def _apply_save_state(self, state: dict[str, Any]) -> None:
        if "player" in state:
            player_state = state["player"]
            inventory_state = state.get("inventory")
            equipment_state = state.get("equipment")
            chest_state = state.get("chests")
            enemy_state = state.get("enemies")
            map_object_state = state.get("map_objects")
        else:
            player_state = state
            inventory_state = None
            equipment_state = None
            chest_state = None
            enemy_state = None
            map_object_state = None

        self.won = bool(state.get("door_unlocked", False))
        self.gold = max(0, int(state.get("gold", STARTING_GOLD)))
        self.load_map(self.map_index_of(state.get("map")))
        self.game_map.load_state(map_object_state)
        self.player.loadState(player_state)

        if inventory_state is None:
            self._seed_inventory()
        else:
            self.inventory.load_state(inventory_state)
            if equipment_state is not None:
                self.equipment.load_state(equipment_state)
            else:
                self.equipment.clear()

        if isinstance(chest_state, dict):
            self.chests.load_state(chest_state)
        else:
            self.chests.reset()

        if isinstance(enemy_state, dict):
            self.enemies.load_state(enemy_state)
        else:
            self.enemies.reset()

        if self.won:
            self.game_map.unlock_door()
        else:
            self.game_map.lock_door()

        self.inventory_screen.inventory_panel.select(0)
        self.inventory_screen.set_gold(self.gold)
        self.update_combat_hud()
        self.inventory_screen.refresh()

    def _enter_game(self) -> None:
        self.currentScreen = "game"
        self.menu.can_resume = True
        self.keys.clear()

    def _seed_inventory(self) -> None:
        self.inventory.clear()
        self.equipment.clear()
        self.inventory.add("apple", 3)
        self.inventory.add("bronze_sword")
        self.inventory.add("stainless_steel_shield")
        self.inventory_screen.inventory_panel.select(0)
        self.inventory_screen.set_gold(self.gold)
        self.update_combat_hud()

    def update_combat_hud(self) -> None:
        attack, defense = get_combat_stats(self.equipment)
        self.atk_value.text = str(attack)
        self.def_value.text = str(defense)

    def draw_combat_hud(self) -> None:
        for label, box_left in (("ATK", self.atk_box_left), ("DEF", self.def_box_left)):
            arcade.draw_lrbt_rectangle_filled(
                box_left,
                box_left + STAT_BOX_SIZE,
                self.stat_bottom,
                self.stat_top,
                STAT_BOX_COLOR,
            )
            arcade.draw_text(
                label,
                box_left + STAT_LABEL_OFFSET,
                (self.stat_top + self.stat_bottom) / 2,
                STAT_TEXT_COLOR,
                font_size=STAT_FONT_SIZE,
                font_name=FONT_NAME,
                anchor_x="left",
                anchor_y="center",
            )

        self.atk_value.draw()
        self.def_value.draw()

    def player_attack(self) -> None:
        if self.player.dead or self.player.moving:
            return

        attack_x, attack_y = self.player.get_attack_position()
        self.attack_effects.append(AttackEffect(attack_x, attack_y))
        self.enemies.attack_at(attack_x, attack_y)

    def on_draw(self) -> None:
        self.clear()

        if self.currentScreen == "menu":
            self.camera.use_gui()
            self.menu.draw()
            return

        if self.currentScreen == "won":
            self.camera.use_gui()
            self.draw_win_screen()
            return

        self.camera.use_world()
        self.game_map.draw(pixelated=True)
        self.enemies.draw(pixelated=True)
        self.player_list.draw(pixelated=True)

        for effect in self.attack_effects:
            effect.draw()

        self.camera.use_gui()
        self.heart_list.draw(pixelated=True)
        self.draw_combat_hud()
        self.inventory_screen.draw()
        self.dialogue.draw()
        self.options.draw()
        self.draw_transition()

    @property
    def box_is_open(self) -> bool:
        return self.dialogue.is_open or self.options.is_open

    def open_fountain(self) -> None:
        self.keys.clear()
        self.pending_choice = "fountain"

        if self.fountain_cooldown > 0:
            self.dialogue.show(FOUNTAIN_WAITING)
            return

        self.options.show_options(FOUNTAIN_PROMPT, ["yes", "no"])

    def drink_fountain(self, choice: str) -> None:
        if choice == "yes":
            self.player.heal(FOUNTAIN_HEAL)
            self.update_health()
            self.fountain_cooldown = FOUNTAIN_COOLDOWN
            self.dialogue.show(FOUNTAIN_DRUNK)
            return

        self.dialogue.show(FOUNTAIN_DECLINED)

    def open_door(self) -> None:
        self.keys.clear()
        self.pending_choice = "door"
        prompt = FINAL_DOOR_PROMPT if self.is_last_map else DOOR_PROMPT
        self.options.show_options(prompt, ["give", "not give"])

    def interact_switch(self, index: int) -> None:
        self.keys.clear()
        self.pending_choice = f"switch:{index}"
        prompt = SWITCH_ON_PROMPT if self.game_map.switch_states[index] else SWITCH_OFF_PROMPT
        self.options.show_options(prompt, ["yes", "no"])

    def interact_pressure_plate(self, index: int) -> None:
        self.keys.clear()
        if self.game_map.pressure_plate_states[index]:
            return

        self.pending_choice = f"pressure_plate:{index}"
        self.options.show_options(PRESSURE_PLATE_PROMPT, ["yes", "no"])

    def handle_switch_choice(self, index: int, choice: str) -> None:
        if choice != "yes":
            return

        state = self.game_map.toggle_switch(index)
        message = SWITCH_ON_MESSAGE if state else SWITCH_OFF_MESSAGE
        self.dialogue.show(message)

    def handle_pressure_plate_choice(self, index: int, choice: str) -> None:
        if choice == "yes":
            self.game_map.activate_pressure_plate(index)

    def interact(self) -> None:
        if self.player.dead or self.player.moving:
            return

        chest_index = self.chests.find_in_front(self.player, self.player.facing)
        if chest_index is not None:
            self.interact_chest(chest_index)
            return

        fake_index = self.chests.find_fake_in_front(
            self.player, self.player.facing
        )
        if fake_index is not None:
            self.interact_fake_chest(fake_index)
            return

        switch_index = self.game_map.switch_index_in_front(
            self.player, self.player.facing
        )
        if switch_index is not None:
            self.interact_switch(switch_index)
            return

        pressure_plate_index = self.game_map.pressure_plate_index_in_front(
            self.player, self.player.facing
        )
        if pressure_plate_index is not None:
            self.interact_pressure_plate(pressure_plate_index)
            return

        if self.game_map.door_is_in_front(self.player, self.player.facing):
            self.open_door()

    def interact_chest(self, chest_index: int) -> None:
        self.keys.clear()
        message = self.chests.interact(chest_index, self.inventory)
        self.inventory_screen.refresh()
        self.dialogue.show(message)

    def interact_fake_chest(self, fake_index: int) -> None:
        self.keys.clear()
        self.dialogue.show(self.chests.interact_fake(fake_index))

    def handle_door_choice(self, choice: str) -> None:
        if choice != "give":
            self.dialogue.show(DOOR_DECLINED)
            return

        if not self.inventory.remove_item("key"):
            self.dialogue.show(DOOR_MISSING_KEY)
            return

        self.inventory_screen.refresh()
        self.game_map.unlock_door()
        self.keys.clear()

        if self.is_last_map:
            self.won = True
            self.currentScreen = "won"
            return

        self.begin_map_transition()

    def draw_win_screen(self) -> None:
        arcade.draw_lrbt_rectangle_filled(
            0,
            self.width,
            0,
            self.height,
            WINDOW_BACKGROUND,
        )
        arcade.draw_text(
            WIN_TITLE,
            self.width / 2,
            self.height / 2 + 70,
            STAT_TEXT_COLOR,
            font_size=WIN_FONT_SIZE,
            font_name=FONT_NAME,
            anchor_x="center",
            anchor_y="center",
        )
        arcade.draw_text(
            WIN_MESSAGE,
            self.width / 2,
            self.height / 2 + 18,
            STAT_TEXT_COLOR,
            font_size=WIN_MESSAGE_FONT_SIZE,
            font_name=FONT_NAME,
            anchor_x="center",
            anchor_y="center",
        )
        arcade.draw_text(
            WIN_HINT,
            self.width / 2,
            self.height / 2 - 32,
            MENU_MUTED_TEXT,
            font_size=WIN_HINT_FONT_SIZE,
            font_name=FONT_NAME,
            anchor_x="center",
            anchor_y="center",
        )

    def update_health(self) -> None:
        for index, heart in enumerate(self.heart_list):
            heart.texture = (
                self.heart_texture
                if self.player.hp > index * HEALTH_PER_HEART
                else self.broken_heart_texture
            )

    def update_hud(self, delta_time: float) -> None:
        moving_input = any(
            key in self.keys
            for key in (arcade.key.W, arcade.key.A, arcade.key.S, arcade.key.D)
        )
        alpha = 100 if moving_input else 255

        hearts_left = self.player.hp // HEALTH_PER_HEART
        if self.player.hp > 0 and self.player.hp % HEALTH_PER_HEART:
            hearts_left += 1

        if hearts_left <= 3:
            self.heart_shake_time += delta_time
            for index, heart in enumerate(self.heart_list):
                heart.center_x = (
                    self._heart_x(index)
                    + math.sin(self.heart_shake_time * 50 + index)
                    * self.heart_shake_strength
                )
                heart.center_y = (
                    self.height - HUD_TOP_OFFSET
                    + math.cos(self.heart_shake_time * 60 + index)
                    * self.heart_shake_strength
                )
                heart.alpha = alpha
        else:
            self.heart_shake_time = 0
            for index, heart in enumerate(self.heart_list):
                heart.center_x = self._heart_x(index)
                heart.center_y = self.height - HUD_TOP_OFFSET
                heart.alpha = alpha

    def _run_box_action(self, symbol: int) -> None:
        if symbol == arcade.key.Z:
            self.dialogue.fast = True
            self.options.fast = True
            return

        if symbol == arcade.key.ESCAPE:
            self.dialogue.close()
            self.options.close()
            return

        if self.options.is_open:
            if symbol == arcade.key.LEFT:
                self.options.move(-1)
                return
            if symbol == arcade.key.RIGHT:
                self.options.move(1)
                return

            if symbol in (arcade.key.ENTER, arcade.key.SPACE):
                if not self.options.is_complete:
                    self.options.finish()
                    return

                choice = self.options.confirm()
                if choice is not None:
                    if self.pending_choice == "door":
                        self.handle_door_choice(choice)
                    elif self.pending_choice.startswith("switch:"):
                        self.handle_switch_choice(
                            int(self.pending_choice.split(":", 1)[1]),
                            choice,
                        )
                    elif self.pending_choice.startswith("pressure_plate:"):
                        self.handle_pressure_plate_choice(
                            int(self.pending_choice.split(":", 1)[1]),
                            choice,
                        )
                    else:
                        self.drink_fountain(choice)
                self.pending_choice = ""
                return

        if symbol in (arcade.key.ENTER, arcade.key.SPACE):
            if not self.dialogue.is_complete:
                self.dialogue.finish()
                return

            self.dialogue.close()

    def _run_inventory_action(self, symbol: int) -> None:
        action = self.inventory_screen.handle_key(symbol)

        if action == "close":
            self.inventory_screen.close()
            self.keys.clear()
        elif isinstance(action, ConfirmResult):
            if action.action == "consume":
                self.player.heal(action.heal)
                self.update_health()
            elif action.action in ("equip", "unequip"):
                self.update_combat_hud()
            elif action.action == "sell":
                self.gold += action.gold
                self.inventory_screen.set_gold(self.gold)

    def _run_menu_action(self, action: str | None) -> None:
        if action == "start":
            self.start_game()
        elif action == "resume":
            self.resume_game()
        elif action == "save":
            self.save_game()
        elif action == "load":
            self.load_game()

    def on_key_press(self, symbol, modifiers) -> None:
        if self.currentScreen == "menu":
            self._run_menu_action(self.menu.handle_key(symbol))
            return

        if self.currentScreen == "won":
            if symbol in (arcade.key.ENTER, arcade.key.SPACE, arcade.key.ESCAPE):
                self.currentScreen = "menu"
                self.menu.can_resume = False
                self.menu.status = ""
            return

        if self.transition_active:
            return

        if self.inventory_screen.is_open:
            self._run_inventory_action(symbol)
            return

        if self.box_is_open:
            self._run_box_action(symbol)
            return

        if symbol == arcade.key.ESCAPE:
            self.pause_game()
            return

        if symbol == arcade.key.I:
            self.inventory_screen.toggle()
            self.keys.clear()
            return

        if symbol == arcade.key.Z:
            self.player_attack()
            return

        if symbol == arcade.key.H:
            self.enemies.spawn_near_player(self.player)
            return

        if symbol in INTERACTION_KEYS:
            self.interact()
            return

        if symbol in (arcade.key.W, arcade.key.A, arcade.key.S, arcade.key.D):
            self.player.set_facing_from_key(symbol)

        self.keys.add(symbol)
    def on_key_release(self, symbol, modifiers) -> None:
        if symbol == arcade.key.Z:
            self.dialogue.fast = False
            self.options.fast = False

        if self.currentScreen == "game":
            self.keys.discard(symbol)

    def on_mouse_motion(self, x, y, dx, dy) -> None:
        if self.currentScreen == "menu":
            self.menu.update_mouse(x, y)

    def on_mouse_press(self, x, y, button, modifiers) -> None:
        if self.currentScreen == "menu":
            self._run_menu_action(self.menu.handle_click(x, y, button))

    def on_update(self, delta_time: float) -> None:
        if self.currentScreen in ("menu", "won"):
            return

        if self.transition_active:
            self.update_transition(delta_time)
            return

        if self.inventory_screen.is_open:
            return

        self.attack_effects = [
            effect
            for effect in self.attack_effects
            if effect.update(delta_time)
        ]

        if self.box_is_open:
            self.dialogue.update(delta_time)
            self.options.update(delta_time)
            return

        if self.fountain_cooldown > 0:
            self.fountain_cooldown = max(
                0.0, self.fountain_cooldown - delta_time
            )

        self.enemies.update(delta_time)
        bumped_fountain = self.player.updatePlayer(self.keys, delta_time)
        self.camera.follow(self.player.center_x, self.player.center_y)
        self.enemies.update_triggers(self.player)

        if self.player.just_finished_move:
            self.enemies.step(self.player)

        if bumped_fountain:
            self.open_fountain()
            return

        switch_index = self.game_map.switch_index_in_collision(
            self.player.lastCollision
        )
        if switch_index is not None:
            self.interact_switch(switch_index)
            return

        pressure_plate_index = self.game_map.pressure_plate_index_in_collision(
            self.player.lastCollision
        )
        if pressure_plate_index is not None:
            self.interact_pressure_plate(pressure_plate_index)
            return

        chest_index = self.chests.find_in_collision(
            self.player.lastCollision,
            self.game_map.chest_colliders,
        )
        if chest_index is not None:
            self.interact_chest(chest_index)
            return

        fake_index = self.chests.find_fake_in_collision(
            self.player.lastCollision,
            self.game_map.fake_chest_colliders,
        )
        if fake_index is not None:
            self.interact_fake_chest(fake_index)
            return

        if self.game_map.door_is_in_collision(self.player.lastCollision):
            self.open_door()
            return

        enemy_damage = self.enemies.contact_damage(self.player)
        if enemy_damage and self.enemy_damage_timer <= 0:
            _, defense = get_combat_stats(self.equipment)
            self.player.takeDamage(enemy_damage, defense)
            self.enemy_damage_timer = self.enemy_damage_cooldown
            self.update_health()
        elif not enemy_damage:
            self.enemy_damage_timer = 0.0
        else:
            self.enemy_damage_timer = max(
                0.0,
                self.enemy_damage_timer - delta_time,
            )

        self.spike_damage_timer -= delta_time

        touching_spikes = arcade.check_for_collision_with_list(
            self.player,
            self.game_map.spikes,
        )
        if touching_spikes and self.spike_damage_timer <= 0:
            _, defense = get_combat_stats(self.equipment)
            self.player.takeDamage(10, defense)
            self.spike_damage_timer = self.spike_damage_cooldown
            self.update_health()
        elif not touching_spikes:
            self.spike_damage_timer = 0

        self.update_hud(delta_time)
