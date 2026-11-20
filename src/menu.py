from pathlib import Path

import arcade

from config import *
from load import load_state


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
    def __init__(self, width: int, height: int, assets_dir: Path):
        self.width = width
        self.height = height
        self.assets_dir = assets_dir
        self.mouse_x = width / 2
        self.mouse_y = height / 2
        self.can_resume = False
        self.status = ""
        self.slot = 1
        self.slot_screen: str | None = None

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

    def resize(self, width: int, height: int) -> None:
        self.width = width
        self.height = height

        title_sprite = self.title[0]
        title_sprite.center_x = (
            width - EDGE_MARGIN - TITLE_ART_WIDTH * TITLE_SCALE / 2
        )
        title_sprite.center_y = (
            height
            - EDGE_MARGIN
            - TITLE_ART_HEIGHT * TITLE_SCALE / 2
            + TITLE_CENTER_OFFSET
        )

        center_x = BUTTON_MARGIN + BUTTON_ART_WIDTH * BUTTON_SCALE / 2
        top = height - EDGE_MARGIN - BUTTON_OFFSET_Y
        step = BUTTON_ART_HEIGHT * BUTTON_SCALE + BUTTON_GAP

        for index, button in enumerate(self.buttons):
            if button.action == "settings":
                button.center_x = (
                    CORNER_MARGIN + SETTINGS_ART_SIZE * SETTINGS_SCALE / 2
                )
                button.center_y = (
                    CORNER_MARGIN + SETTINGS_ART_SIZE * SETTINGS_SCALE / 2
                )
                continue

            button.center_x = center_x
            button.center_y = (
                top
                - BUTTON_ART_HEIGHT * BUTTON_SCALE / 2
                - index * step
            )

        self.mouse_x = min(self.mouse_x, width)
        self.mouse_y = min(self.mouse_y, height)

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

    def change_slot(self, delta: int) -> None:
        self.slot = (self.slot - 1 + delta) % SAVE_SLOTS + 1
        self.status = ""

    def open_slot_screen(self, mode: str) -> None:
        self.slot_screen = mode
        self.status = ""

    def _slot_layout(self) -> tuple[float, float, list[float]]:
        rows_height = SAVE_SLOTS * SLOT_MENU_ROW_HEIGHT
        rows_top = self.height / 2 + rows_height / 2
        top = rows_top + SLOT_MENU_TITLE_SIZE * 2
        left = (self.width - SLOT_MENU_WIDTH) / 2
        first_center = rows_top - SLOT_MENU_ROW_HEIGHT / 2
        centers = [
            first_center - i * SLOT_MENU_ROW_HEIGHT
            for i in range(SAVE_SLOTS)
        ]
        return left, top, centers

    def _slot_at(self, y: float) -> int | None:
        _, _, centers = self._slot_layout()
        for index, center_y in enumerate(centers):
            if abs(y - center_y) <= SLOT_MENU_ROW_HEIGHT / 2:
                return index + 1
        return None

    def _slot_action(self) -> str:
        return f"{self.slot_screen}_slot"

    def handle_key(self, symbol: int) -> str | None:
        if self.slot_screen is not None:
            if arcade.key.KEY_1 <= symbol <= arcade.key.KEY_9:
                slot = symbol - arcade.key.KEY_1 + 1
                if slot <= SAVE_SLOTS:
                    self.slot = slot
                return None
            if symbol in (arcade.key.UP, arcade.key.W):
                self.change_slot(-1)
                return None
            if symbol in (arcade.key.DOWN, arcade.key.S):
                self.change_slot(1)
                return None
            if symbol in (arcade.key.ENTER, arcade.key.SPACE):
                return self._slot_action()
            if symbol == arcade.key.ESCAPE:
                self.slot_screen = None
                return None
            return None

        if arcade.key.KEY_1 <= symbol <= arcade.key.KEY_9:
            slot = symbol - arcade.key.KEY_1 + 1
            if slot <= SAVE_SLOTS:
                self.slot = slot
                self.status = ""
                return None
        if symbol == arcade.key.LEFT:
            self.change_slot(-1)
            return None
        if symbol == arcade.key.RIGHT:
            self.change_slot(1)
            return None
        if symbol in (arcade.key.ENTER, arcade.key.SPACE):
            return "resume" if self.can_resume else "start"
        return None

    def handle_click(self, x: float, y: float, button: int) -> str | None:
        if button != arcade.MOUSE_BUTTON_LEFT:
            return None

        self.update_mouse(x, y)

        if self.slot_screen is not None:
            slot = self._slot_at(y)
            if slot is not None:
                self.slot = slot
                return self._slot_action()
            self.slot_screen = None
            return None

        hovered = self._hovered_button()
        if hovered is None:
            return None

        if hovered.action in ("load", "save"):
            self.open_slot_screen(hovered.action)
            return None
        if hovered.action == "start" and self.can_resume:
            return "resume"
        return hovered.action

    def _draw_slot_screen(self) -> None:
        left, top, centers = self._slot_layout()
        right = left + SLOT_MENU_WIDTH
        bottom = centers[-1] - SLOT_MENU_ROW_HEIGHT / 2
        title = (
            SLOT_LOAD_TITLE
            if self.slot_screen == "load"
            else SLOT_SAVE_TITLE
        )

        arcade.draw_lrbt_rectangle_filled(
            left - 20,
            right + 20,
            bottom - 50,
            top + 20,
            PANEL_COLOR,
        )
        arcade.draw_lrbt_rectangle_outline(
            left - 20,
            right + 20,
            bottom - 50,
            top + 20,
            INV_BORDER_COLOR,
            2,
        )
        arcade.draw_text(
            title,
            self.width / 2,
            top - SLOT_MENU_TITLE_SIZE,
            INV_ACCENT_COLOR,
            font_size=SLOT_MENU_TITLE_SIZE,
            font_name=FONT_NAME,
            anchor_x="center",
            anchor_y="center",
        )

        for index, center_y in enumerate(centers):
            slot = index + 1
            row_bottom = center_y - SLOT_MENU_ROW_HEIGHT / 2
            selected = slot == self.slot
            arcade.draw_lrbt_rectangle_filled(
                left,
                right,
                row_bottom,
                row_bottom + SLOT_MENU_ROW_HEIGHT,
                SLOT_SELECTED_COLOR if selected else SLOT_COLOR,
            )
            arcade.draw_lrbt_rectangle_outline(
                left,
                right,
                row_bottom,
                row_bottom + SLOT_MENU_ROW_HEIGHT,
                INV_SELECTED_BORDER if selected else INV_SLOT_BORDER,
                2 if selected else 1,
            )

            state = load_state(save_slot_path(slot))
            if state is None:
                detail = "EMPTY"
            else:
                detail = f"{state.get('map', 'unknown')}  -  {state.get('gold', 0)}g"
            arcade.draw_text(
                f"SLOT {slot}   {detail}",
                left + 16,
                center_y,
                INV_ACCENT_COLOR if selected else TEXT_COLOR,
                font_size=SLOT_MENU_ROW_SIZE,
                font_name=FONT_NAME,
                anchor_x="left",
                anchor_y="center",
            )

        arcade.draw_text(
            "UP/DOWN or 1-6 select   ENTER confirm   ESC back",
            self.width / 2,
            bottom - 26,
            MENU_MUTED_TEXT,
            font_size=SLOT_MENU_HINT_SIZE,
            font_name=FONT_NAME,
            anchor_x="center",
            anchor_y="center",
        )
        if self.status:
            arcade.draw_text(
                self.status,
                self.width / 2,
                bottom - 44,
                MENU_MUTED_TEXT,
                font_size=SLOT_MENU_HINT_SIZE,
                font_name=FONT_NAME,
                anchor_x="center",
                anchor_y="center",
            )

    def draw(self) -> None:
        arcade.draw_lrbt_rectangle_filled(
            0, self.width, 0, self.height, MENU_BACKGROUND
        )

        if self.slot_screen is not None:
            self._draw_slot_screen()
            return

        self.title.draw(pixelated=True)
        self.buttons.draw(pixelated=True)

        title_sprite = self.title[0]
        title_center_x = title_sprite.center_x
        arcade.draw_text(
            "ENTER or SPACE to continue" if self.can_resume
            else "ENTER or SPACE to start",
            title_center_x,
            110,
            MENU_MUTED_TEXT,
            font_size=HINT_SIZE,
            font_name=FONT_NAME,
            anchor_x="center",
            anchor_y="center",
        )
        arcade.draw_text(
            "WASD MOVE   E INTERACT   Z ATTACK   H DAMAGE TEST",
            title_center_x,
            74,
            MENU_MUTED_TEXT,
            font_size=CONTROLS_SIZE,
            font_name=FONT_NAME,
            anchor_x="center",
            anchor_y="center",
        )
        arcade.draw_text(
            "I INVENTORY   TAB EQUIPMENT   ESC MENU   Z SKIP TEXT",
            title_center_x,
            56,
            MENU_MUTED_TEXT,
            font_size=CONTROLS_SIZE,
            font_name=FONT_NAME,
            anchor_x="center",
            anchor_y="center",
        )

        if self.status:
            arcade.draw_text(
                self.status,
                BUTTON_MARGIN,
                160,
                MENU_MUTED_TEXT,
                font_size=STATUS_SIZE,
                font_name=FONT_NAME,
                anchor_x="left",
                anchor_y="center",
            )

