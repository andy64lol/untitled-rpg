import arcade

from font import FONT_NAME
from paths import DIALOGUE_BOX_PATH

BOX_SCALE = 2
BOX_MARGIN = 40
TEXT_MARGIN = 40
TEXT_SIZE = 18
TEXT_SPEED = 45
FAST_SPEED_MULTIPLIER = 6


class DialogueBox:

    TEXT_COLOR = (235, 232, 213)

    def __init__(self, width: int, height: int):
        box = arcade.Sprite(
            arcade.load_texture(str(DIALOGUE_BOX_PATH)),
            scale=BOX_SCALE,
            pixelated=True,
        )
        box.center_x = width / 2
        box.center_y = BOX_MARGIN + box.height / 2
        self.box = arcade.SpriteList()
        self.box.append(box)

        self.left = box.left
        self.top = box.top
        self.bottom = box.bottom
        self.text_width = int(box.width) - TEXT_MARGIN * 2

        self.message = arcade.Text(
            "",
            self.left + TEXT_MARGIN,
            self.top - TEXT_MARGIN,
            self.TEXT_COLOR,
            font_size=TEXT_SIZE,
            font_name=FONT_NAME,
            width=self.text_width,
            anchor_x="left",
            anchor_y="top",
            multiline=True,
        )
        self.is_open = False
        self.fast = False
        self.full_text = ""
        self.revealed = 0.0

    @property
    def is_complete(self) -> bool:
        return self.revealed >= len(self.full_text)

    def show(self, message: str) -> None:
        self.full_text = message
        self.revealed = 0.0
        self.message.text = ""
        self.is_open = True

    def update(self, delta_time: float) -> None:
        if not self.is_open or self.is_complete:
            return

        speed = TEXT_SPEED
        if self.fast:
            speed *= FAST_SPEED_MULTIPLIER

        self.revealed = min(
            len(self.full_text), self.revealed + speed * delta_time
        )
        self.message.text = self.full_text[:int(self.revealed)]

    def finish(self) -> None:
        self.revealed = float(len(self.full_text))
        self.message.text = self.full_text

    def close(self) -> None:
        self.is_open = False

    def draw(self) -> None:
        if not self.is_open:
            return

        self.box.draw(pixelated=True)
        self.message.draw()


class OptionBox(DialogueBox):

    def __init__(self, width: int, height: int):
        super().__init__(width, height)
        self.options: list[str] = []
        self.index = 0

        self.option_line = arcade.Text(
            "",
            self.left + TEXT_MARGIN,
            self.bottom + TEXT_MARGIN,
            self.TEXT_COLOR,
            font_size=TEXT_SIZE,
            font_name=FONT_NAME,
            width=self.text_width,
            anchor_x="left",
            anchor_y="bottom",
            multiline=True,
        )

    def show_options(self, message: str, options: list[str]) -> None:
        self.show(message)
        self.options = list(options)
        self.index = 0
        self._render_options()

    def move(self, delta: int) -> None:
        if not self.options:
            return

        self.index = (self.index + delta) % len(self.options)
        self._render_options()

    def confirm(self) -> str | None:
        if not self.is_open:
            return None

        choice = self.options[self.index] if self.options else None
        self.close()
        return choice

    def draw(self) -> None:
        super().draw()

        if self.is_open and self.options and self.is_complete:
            self.option_line.draw()

    def _render_options(self) -> None:
        parts = [
            f"> {option}" if index == self.index else option
            for index, option in enumerate(self.options)
        ]
        self.option_line.text = "  ".join(parts)
