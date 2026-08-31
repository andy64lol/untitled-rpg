import math
from dataclasses import dataclass

import arcade
from camera import CameraController
from config import *
from dialogue import DialogueBox, OptionBox
from font import FONT_NAME
from inventory import ConfirmResult, Equipment, Inventory, InventoryScreen
from items import get_combat_stats
from load import load_state
from map import GameMap
from menu import MainMenu
from paths import (
    ASSETS_DIR,
    BROKEN_HEART_PATH,
    HEART_PATH,
    MAPS_DIR,
    SAVE_PATH,
)
import save
from sprites import Player


@dataclass
class AttackEffect:
    center_x: float
    center_y: float
    timer: float = 0.15
    size: float = 32

    def update(self, delta_time: float) -> bool:
        self.timer -= delta_time
        return self.timer > 0

    def draw(self) -> None:
        half = self.size / 2
        alpha = max(0, min(255, int(255 * (self.timer / 0.15))))
        arcade.draw_lrbt_rectangle_filled(
            self.center_x - half,
            self.center_x + half,
            self.center_y - half,
            self.center_y + half,
            (255, 220, 120, alpha),
        )


class GameWindow(arcade.Window):

    WIDTH = 1280
    HEIGHT = 720

    FOUNTAIN_HEAL = 20
    FOUNTAIN_COOLDOWN = 60.0
    FOUNTAIN_PROMPT = "A regenerating fountain, drink from here?"
    FOUNTAIN_DRUNK = (
        "You drank from the fountain and got refreshed! (2 hearts healed)"
    )
    FOUNTAIN_DECLINED = "You chose not to drink."
    FOUNTAIN_WAITING = "Don't drink too much water!"

    STAT_BOX_COLOR = (72, 76, 88)
    STAT_TEXT_COLOR = (235, 232, 213)

    def __init__(self):
        super().__init__(self.WIDTH, self.HEIGHT, "Untitled RPG", resizable=True)
        self.set_minimum_size(960, 540)
        arcade.set_background_color((10, 12, 20, 255))

        self.camera = CameraController(self.width, self.height)
        self.menu = MainMenu(self.width, self.height, ASSETS_DIR)
        self.currentScreen = "menu"
        self.keys: set[int] = set()
        self.stat_top = 0
        self.stat_bottom = 0

        self.game_map = GameMap(MAPS_DIR / "map1.tmx")
        self.player = Player(
            self.game_map.tilemap,
            self.game_map.collision,
            self.game_map.fountains,
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

        self.inventory = Inventory()
        self.equipment = Equipment()
        self.inventory_screen = InventoryScreen(
            self.width, self.height, self.inventory, self.equipment
        )
        self._seed_inventory()

        self.attack_effects: list[AttackEffect] = []
        self.heart_shake_time = 0
        self.heart_shake_strength = 2
        self.spike_damage_cooldown = 1.0
        self.spike_damage_timer = 0

    def on_resize(self, width: int, height: int) -> None:
        super().on_resize(width, height)
        self.camera.resize(width, height)
        self.menu.resize(width, height)
        self.dialogue.resize(width, height)
        self.options.resize(width, height)
        self.inventory_screen.resize(width, height)
        self._layout_health_hud(height)
        self._layout_combat_hud(height)

        if self.currentScreen == "game":
            self.camera.follow(self.player.center_x, self.player.center_y)

    def _create_health_hud(self) -> None:
        heart_size = 30
        heart_count = self.player.maxHp // 10

        for index in range(heart_count):
            heart = arcade.Sprite(self.heart_texture, scale=1, pixelated=True)
            heart.width = heart_size
            heart.height = heart_size
            self.heart_list.append(heart)

        self._layout_health_hud(self.height)

    def _layout_health_hud(self, height: int | None = None) -> None:
        hud_height = height if height is not None else self.height
        heart_size = 30
        for index, heart in enumerate(self.heart_list):
            heart.center_x = 20 + index * heart_size
            heart.center_y = hud_height - 20

    def _create_combat_hud(self) -> None:
        self.atk_box_left = 20
        self.def_box_left = 78

        self.atk_value = arcade.Text(
            "1",
            0,
            0,
            self.STAT_TEXT_COLOR,
            font_size=12,
            font_name=FONT_NAME,
            anchor_x="center",
            anchor_y="center",
        )
        self.def_value = arcade.Text(
            "1",
            0,
            0,
            self.STAT_TEXT_COLOR,
            font_size=12,
            font_name=FONT_NAME,
            anchor_x="center",
            anchor_y="center",
        )
        self._layout_combat_hud()

    def _layout_combat_hud(self) -> None:
        self.stat_top = self.height - 34
        self.stat_bottom = self.height - 58
        center_y = (self.stat_top + self.stat_bottom) / 2

        self.atk_value.x = self.atk_box_left + 34
        self.atk_value.y = center_y
        self.def_value.x = self.def_box_left + 34
        self.def_value.y = center_y

    def start_game(self) -> None:
        self.player.reset()
        self._seed_inventory()
        self.update_health()
        self.update_combat_hud()
        self.camera.follow(self.player.center_x, self.player.center_y)
        self._enter_game()

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
        self._enter_game()

    def _build_save_state(self) -> dict:
        return {
            "player": self.player.saveState(),
            "inventory": self.inventory.save_state(),
            "equipment": self.equipment.save_state(),
        }

    def _apply_save_state(self, state: dict) -> None:
        if "player" in state:
            player_state = state["player"]
            inventory_state = state.get("inventory")
            equipment_state = state.get("equipment")
        else:
            player_state = state
            inventory_state = None
            equipment_state = None

        self.player.loadState(player_state)

        if inventory_state is not None:
            self.inventory.load_state(inventory_state)
        else:
            self._seed_inventory()
            return

        if equipment_state is not None:
            self.equipment.load_state(equipment_state)
        else:
            self.equipment.clear()

        self.inventory_screen.inventory_panel.select(0)
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
        self.update_combat_hud()

    def update_combat_hud(self) -> None:
        attack, defense = get_combat_stats(self.equipment)
        self.atk_value.text = str(attack)
        self.def_value.text = str(defense)

    def draw_combat_hud(self) -> None:
        for label, box_left in (("ATK", self.atk_box_left), ("DEF", self.def_box_left)):
            arcade.draw_lrbt_rectangle_filled(
                box_left,
                box_left + 52,
                self.stat_bottom,
                self.stat_top,
                self.STAT_BOX_COLOR,
            )
            arcade.draw_text(
                label,
                box_left + 6,
                (self.stat_top + self.stat_bottom) / 2,
                self.STAT_TEXT_COLOR,
                font_size=10,
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

    def on_draw(self) -> None:
        self.clear()

        if self.currentScreen == "menu":
            self.menu.draw()
            return

        self.camera.use_world()
        self.game_map.draw(pixelated=True)
        self.player_list.draw(pixelated=True)

        for effect in self.attack_effects:
            effect.draw()

        self.camera.use_gui()
        self.heart_list.draw(pixelated=True)
        self.draw_combat_hud()
        self.inventory_screen.draw()
        self.dialogue.draw()
        self.options.draw()

    @property
    def box_is_open(self) -> bool:
        return self.dialogue.is_open or self.options.is_open

    def open_fountain(self) -> None:
        self.keys.clear()

        if self.fountain_cooldown > 0:
            self.dialogue.show(self.FOUNTAIN_WAITING)
            return

        self.options.show_options(self.FOUNTAIN_PROMPT, ["yes", "no"])

    def drink_fountain(self, choice: str) -> None:
        if choice == "yes":
            self.player.heal(self.FOUNTAIN_HEAL)
            self.update_health()
            self.fountain_cooldown = self.FOUNTAIN_COOLDOWN
            self.dialogue.show(self.FOUNTAIN_DRUNK)
            return

        self.dialogue.show(self.FOUNTAIN_DECLINED)

    def update_health(self) -> None:
        for index, heart in enumerate(self.heart_list):
            heart.texture = (
                self.heart_texture
                if self.player.hp > index * 10
                else self.broken_heart_texture
            )

    def update_hud(self, delta_time: float) -> None:
        moving_input = any(
            key in self.keys
            for key in (arcade.key.W, arcade.key.A, arcade.key.S, arcade.key.D)
        )
        alpha = 100 if moving_input else 255

        hearts_left = self.player.hp // 10
        if self.player.hp > 0 and self.player.hp % 10:
            hearts_left += 1

        if hearts_left <= 3:
            self.heart_shake_time += delta_time
            for index, heart in enumerate(self.heart_list):
                heart.center_x = (
                    20 + index * 32
                    + math.sin(self.heart_shake_time * 50 + index)
                    * self.heart_shake_strength
                )
                heart.center_y = (
                    self.height - 20
                    + math.cos(self.heart_shake_time * 60 + index)
                    * self.heart_shake_strength
                )
                heart.alpha = alpha
        else:
            self.heart_shake_time = 0
            for index, heart in enumerate(self.heart_list):
                heart.center_x = 20 + index * 32
                heart.center_y = self.height - 20
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
                    self.drink_fountain(choice)
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

        if symbol in (arcade.key.W, arcade.key.A, arcade.key.S, arcade.key.D):
            self.player.set_facing_from_key(symbol)

        self.keys.add(symbol)
        if symbol == arcade.key.H:
            _, defense = get_combat_stats(self.equipment)
            self.player.takeDamage(10, defense)
            self.update_health()

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
        if self.currentScreen == "menu":
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

        bumped_fountain = self.player.updatePlayer(self.keys, delta_time)
        self.camera.follow(self.player.center_x, self.player.center_y)

        if bumped_fountain:
            self.open_fountain()
            return

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
