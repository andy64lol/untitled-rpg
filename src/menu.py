from pathlib import Path

import arcade

from font import FONT_NAME
from paths import TITLE_PATH

TITLE_SCALE = 2
TITLE_ART_WIDTH = 240
TITLE_ART_HEIGHT = 108
TITLE_CENTER_OFFSET = 4
EDGE_MARGIN = 40

BUTTON_SCALE = 1
BUTTON_ART_WIDTH = 64
BUTTON_ART_HEIGHT = 32
BUTTON_MARGIN = 120
BUTTON_GAP = 32
BUTTON_OFFSET_Y = 300

SETTINGS_SCALE = 1
SETTINGS_ART_SIZE = 64
CORNER_MARGIN = 20

HINT_SIZE = 10
CONTROLS_SIZE = 10
STATUS_SIZE = 10


class MenuButton(arcade.Sprite):

    def __init__(
        self,
        action: str,
        normal: arcade.Texture,
        hover: arcade.Texture,
        scale: int,
        center_x: float,
        center_y: float,
    ):
        super().__init__(
            normal,
            scale=scale,
            center_x=center_x,
            center_y=center_y,
            pixelated=True,
        )
        self.action = action
        self.normal_texture = normal
        self.hover_texture = hover


class MainMenu:

    BACKGROUND = (10, 12, 20)
    MUTED_TEXT = (142, 151, 164)

    def __init__(self, width: int, height: int, assets_dir: Path):
        self.width = width
        self.height = height
        self.assets_dir = assets_dir
        self.mouse_x = width / 2
        self.mouse_y = height / 2
        self.can_resume = False
        self.status = ""

        title_texture = arcade.load_texture(str(TITLE_PATH))
        title_sprite = arcade.Sprite(title_texture, scale=TITLE_SCALE, pixelated=True)
        title_sprite.center_x = (
            width - EDGE_MARGIN - TITLE_ART_WIDTH * TITLE_SCALE / 2
        )
        title_sprite.center_y = (
            height
            - EDGE_MARGIN
            - TITLE_ART_HEIGHT * TITLE_SCALE / 2
            + TITLE_CENTER_OFFSET
        )
        self.title = arcade.SpriteList()
        self.title.append(title_sprite)

        self.buttons = self._create_buttons(assets_dir / "UI" / "buttons")

    def _create_buttons(self, buttons_dir: Path) -> arcade.SpriteList[MenuButton]:
        center_x = BUTTON_MARGIN + BUTTON_ART_WIDTH * BUTTON_SCALE / 2
        top = self.height - EDGE_MARGIN - BUTTON_OFFSET_Y
        step = BUTTON_ART_HEIGHT * BUTTON_SCALE + BUTTON_GAP

        column = (("start", "start"), ("load", "load"), ("save", "save"))
        buttons: arcade.SpriteList[MenuButton] = arcade.SpriteList()

        for index, (name, action) in enumerate(column):
            normal = arcade.load_texture(str(buttons_dir / f"{name}.png"))
            hover = arcade.load_texture(str(buttons_dir / f"{name}_hover.png"))
            buttons.append(
                MenuButton(
                    action,
                    normal,
                    hover,
                    BUTTON_SCALE,
                    center_x,
                    top - BUTTON_ART_HEIGHT * BUTTON_SCALE / 2 - index * step,
                )
            )

        settings_normal = arcade.load_texture(str(buttons_dir / "settings.png"))
        settings_hover = arcade.load_texture(
            str(buttons_dir / "settings_hover.png")
        )
        buttons.append(
            MenuButton(
                "settings",
                settings_normal,
                settings_hover,
                SETTINGS_SCALE,
                CORNER_MARGIN + SETTINGS_ART_SIZE * SETTINGS_SCALE / 2,
                CORNER_MARGIN + SETTINGS_ART_SIZE * SETTINGS_SCALE / 2,
            )
        )

        return buttons

    def _hovered_button(self) -> MenuButton | None:
        return next(
            (
                button
                for button in self.buttons
                if button.left <= self.mouse_x <= button.right
                and button.bottom <= self.mouse_y <= button.top
            ),
            None,
        )

    def update_mouse(self, x: float, y: float) -> None:
        self.mouse_x = x
        self.mouse_y = y
        self.status = ""

        hovered = self._hovered_button()
        for button in self.buttons:
            button.texture = (
                button.hover_texture if button is hovered else button.normal_texture
            )

    def handle_key(self, symbol: int) -> str | None:
        if symbol in (arcade.key.ENTER, arcade.key.SPACE):
            return "resume" if self.can_resume else "start"
        return None

    def handle_click(self, x: float, y: float, button: int) -> str | None:
        if button != arcade.MOUSE_BUTTON_LEFT:
            return None

        self.update_mouse(x, y)
        hovered = self._hovered_button()
        if hovered is None:
            return None

        if hovered.action == "start" and self.can_resume:
            return "resume"
        return hovered.action

    def draw(self) -> None:
        arcade.draw_lrbt_rectangle_filled(
            0, self.width, 0, self.height, self.BACKGROUND
        )

        self.title.draw(pixelated=True)
        self.buttons.draw(pixelated=True)

        title_left = self.width - EDGE_MARGIN - TITLE_ART_WIDTH * TITLE_SCALE
        arcade.draw_text(
            "ENTER or SPACE to continue" if self.can_resume
            else "ENTER or SPACE to start",
            title_left,
            110,
            self.MUTED_TEXT,
            font_size=HINT_SIZE,
            font_name=FONT_NAME,
            anchor_x="left",
            anchor_y="center",
        )
        arcade.draw_text(
            "WASD  MOVE        H  DAMAGE TEST        I  INVENTORY"
            "        TAB  EQUIPMENT        ESC  MENU        Z  SKIP TEXT",
            title_left,
            70,
            self.MUTED_TEXT,
            font_size=CONTROLS_SIZE,
            font_name=FONT_NAME,
            anchor_x="left",
            anchor_y="center",
        )

        if self.status:
            arcade.draw_text(
                self.status,
                BUTTON_MARGIN,
                160,
                self.MUTED_TEXT,
                font_size=STATUS_SIZE,
                font_name=FONT_NAME,
                anchor_x="left",
                anchor_y="center",
            )
