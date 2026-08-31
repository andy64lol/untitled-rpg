import math
import arcade
from camera import CameraController
from config import *
from dialogue import DialogueBox, OptionBox
from inventory import COLUMNS, Equipment, Inventory, InventoryScreen
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

    def __init__(self):
        super().__init__(self.WIDTH, self.HEIGHT, "Untitled RPG")
        arcade.set_background_color((10, 12, 20, 255))

        self.camera = CameraController(self.width, self.height)
        self.menu = MainMenu(self.width, self.height, ASSETS_DIR)
        self.currentScreen = "menu"
        self.keys: set[int] = set()

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

        self.dialogue = DialogueBox(self.width, self.height)
        self.options = OptionBox(self.width, self.height)
        self.fountain_cooldown = 0.0

        self.inventory = Inventory()
        self.equipment = Equipment()
        self.inventory_screen = InventoryScreen(
            self.width, self.height, self.inventory, self.equipment
        )
        self._seed_inventory()

        self.heart_shake_time = 0
        self.heart_shake_strength = 2
        self.spike_damage_cooldown = 1.0
        self.spike_damage_timer = 0

    def _create_health_hud(self) -> None:
        heart_size = 30
        heart_count = self.player.maxHp // 10

        for index in range(heart_count):
            heart = arcade.Sprite(self.heart_texture, scale=1, pixelated=True)
            heart.center_x = 20 + index * heart_size
            heart.center_y = self.height - 20
            heart.width = heart_size
            heart.height = heart_size
            self.heart_list.append(heart)

    def start_game(self) -> None:
        self.player.reset()
        self._seed_inventory()
        self.update_health()
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
        save.save_state(SAVE_PATH, self.player.saveState())
        self.menu.status = "Game saved."
        self.menu.can_resume = True

    def load_game(self) -> None:
        state = load_state(SAVE_PATH)
        if state is None:
            self.menu.status = "No saved game found."
            return

        self.player.loadState(state)
        self.update_health()
        self.camera.follow(self.player.center_x, self.player.center_y)
        self._enter_game()

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

    def on_draw(self) -> None:
        self.clear()

        if self.currentScreen == "menu":
            self.menu.draw()
            return

        self.camera.use_world()
        self.game_map.draw()
        self.player_list.draw(pixelated=True)

        self.camera.use_gui()
        self.heart_list.draw(pixelated=True)
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
        if symbol in (arcade.key.I, arcade.key.ESCAPE):
            self.inventory_screen.close()
            self.keys.clear()
            return

        if symbol == arcade.key.TAB:
            self.inventory_screen.switch_focus()
            return

        if symbol in (arcade.key.ENTER, arcade.key.SPACE):
            self.inventory_screen.confirm()
            return

        if self.inventory_screen.focus == "equipment":
            if symbol == arcade.key.UP:
                self.inventory_screen.move(-1)
            elif symbol == arcade.key.DOWN:
                self.inventory_screen.move(1)
            return

        if symbol == arcade.key.LEFT:
            self.inventory_screen.move(-1)
        elif symbol == arcade.key.RIGHT:
            self.inventory_screen.move(1)
        elif symbol == arcade.key.UP:
            self.inventory_screen.move(-COLUMNS)
        elif symbol == arcade.key.DOWN:
            self.inventory_screen.move(COLUMNS)

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

        self.keys.add(symbol)
        if symbol == arcade.key.H:
            self.player.takeDamage(10)
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
            self.player.takeDamage(10)
            self.spike_damage_timer = self.spike_damage_cooldown
            self.update_health()
        elif not touching_spikes:
            self.spike_damage_timer = 0

        self.update_hud(delta_time)
