from dataclasses import dataclass

import arcade
from PIL import ImageFilter

from font import FONT_NAME
from items import ITEMS
from paths import EQUIPMENT_SLOTS_DIR

COLUMNS = 4
ROWS = 3
SLOT_SIZE = 48
SLOT_GAP = 8
ICON_SCALE = 4
MAX_STACK = 16

EQUIPMENT_SLOTS = ("weapon", "offhand", "armor")
EQUIPMENT_SLOT_SIZE = 64
EQUIPMENT_SLOT_GAP = 8

PANEL_PADDING = 20
PANEL_GAP = 24
LABEL_HEIGHT = 18

PANEL_COLOR = (16, 18, 28, 240)
SLOT_COLOR = (38, 42, 58)
SLOT_SELECTED_COLOR = (96, 108, 148)
EQUIPMENT_SLOT_BG = (72, 76, 88)
TEXT_COLOR = (235, 232, 213)
LABEL_SIZE = 10
COUNT_SIZE = 10


def _load_slot_textures() -> tuple[dict[str, arcade.Texture], dict[str, arcade.Texture]]:
    normal: dict[str, arcade.Texture] = {}
    blurred: dict[str, arcade.Texture] = {}

    for slot in EQUIPMENT_SLOTS:
        texture = arcade.load_texture(str(EQUIPMENT_SLOTS_DIR / f"{slot}.png"))
        normal[slot] = texture
        blurred[slot] = arcade.Texture(
            image=texture.image.filter(ImageFilter.GaussianBlur(radius=2))
        )

    return normal, blurred


SLOT_TEXTURES, BLURRED_SLOT_TEXTURES = _load_slot_textures()


@dataclass
class Stack:
    item_id: str
    count: int = 1


@dataclass(frozen=True)
class ConfirmResult:
    action: str
    heal: int = 0


class Inventory:
    def __init__(self, size: int = COLUMNS * ROWS):
        self.slots: list[Stack | None] = [None] * size

    def clear(self) -> None:
        self.slots = [None] * len(self.slots)

    def add(self, item_id: str, count: int = 1) -> bool:
        remaining = count

        for stack in self.slots:
            if stack is None or stack.item_id != item_id:
                continue

            space = MAX_STACK - stack.count
            if space <= 0:
                continue

            taken = min(space, remaining)
            stack.count += taken
            remaining -= taken

            if remaining == 0:
                return True

        for index, stack in enumerate(self.slots):
            if stack is not None:
                continue

            taken = min(MAX_STACK, remaining)
            self.slots[index] = Stack(item_id, taken)
            remaining -= taken

            if remaining == 0:
                return True

        return False

    def remove_at(self, index: int, count: int = 1) -> tuple[str, int] | None:
        stack = self.slots[index]
        if stack is None:
            return None

        item_id = stack.item_id
        removed = min(count, stack.count)
        stack.count -= removed

        if stack.count == 0:
            self.slots[index] = None

        return item_id, removed

    def save_state(self) -> dict:
        return {
            "slots": [
                None
                if stack is None
                else {"item_id": stack.item_id, "count": stack.count}
                for stack in self.slots
            ]
        }

    def load_state(self, state: dict) -> None:
        saved_slots = state.get("slots", [])
        self.clear()

        for index, entry in enumerate(saved_slots):
            if index >= len(self.slots) or entry is None:
                continue

            item_id = entry.get("item_id")
            count = int(entry.get("count", 1))
            if item_id in ITEMS and count > 0:
                self.slots[index] = Stack(item_id, count)


class Equipment:
    def __init__(self):
        self.slots: dict[str, str | None] = {
            slot: None for slot in EQUIPMENT_SLOTS
        }

    def clear(self) -> None:
        for slot in EQUIPMENT_SLOTS:
            self.slots[slot] = None

    def equip(self, slot: str, item_id: str) -> str | None:
        previous = self.slots[slot]
        self.slots[slot] = item_id
        return previous

    def unequip(self, slot: str) -> str | None:
        item_id = self.slots[slot]
        self.slots[slot] = None
        return item_id

    def save_state(self) -> dict:
        return {"slots": dict(self.slots)}

    def load_state(self, state: dict) -> None:
        self.clear()
        saved_slots = state.get("slots", {})

        for slot in EQUIPMENT_SLOTS:
            item_id = saved_slots.get(slot)
            if item_id in ITEMS:
                self.slots[slot] = item_id


class EquipmentPanel:
    def __init__(self, left: float, top: float, equipment: Equipment):
        self.equipment = equipment
        self.index = 0

        grid_height = (
            len(EQUIPMENT_SLOTS) * EQUIPMENT_SLOT_SIZE
            + (len(EQUIPMENT_SLOTS) - 1) * EQUIPMENT_SLOT_GAP
        )
        self.panel_width = EQUIPMENT_SLOT_SIZE + PANEL_PADDING * 2
        self.panel_height = grid_height + PANEL_PADDING * 2 + LABEL_HEIGHT

        self.left = left
        self.top = top
        self.bottom = self.top - self.panel_height

        self.slot_icons = arcade.SpriteList()
        self.icons = arcade.SpriteList()
        self.highlights: list[tuple[float, float, float, float]] = []

        for index, slot in enumerate(EQUIPMENT_SLOTS):
            slot_left = self._slot_left()
            bottom = self._slot_bottom(index)

            self.slot_icons.append(
                arcade.Sprite(
                    SLOT_TEXTURES[slot],
                    center_x=slot_left + EQUIPMENT_SLOT_SIZE / 2,
                    center_y=bottom + EQUIPMENT_SLOT_SIZE / 2,
                    pixelated=True,
                )
            )
            self.icons.append(
                arcade.Sprite(
                    ITEMS["apple"].texture,
                    scale=ICON_SCALE,
                    center_x=slot_left + EQUIPMENT_SLOT_SIZE / 2,
                    center_y=bottom + EQUIPMENT_SLOT_SIZE / 2,
                    pixelated=True,
                )
            )
            self.highlights.append(
                (
                    slot_left,
                    slot_left + EQUIPMENT_SLOT_SIZE,
                    bottom,
                    bottom + EQUIPMENT_SLOT_SIZE,
                )
            )

        self.label = arcade.Text(
            "",
            self.left + PANEL_PADDING,
            self.bottom + LABEL_HEIGHT / 2,
            TEXT_COLOR,
            font_size=LABEL_SIZE,
            font_name=FONT_NAME,
            width=self.panel_width - PANEL_PADDING * 2,
            anchor_x="left",
            anchor_y="center",
        )
        self.refresh()

    @property
    def selected_slot(self) -> str:
        return EQUIPMENT_SLOTS[self.index]

    @property
    def selected_item_id(self) -> str | None:
        return self.equipment.slots[self.selected_slot]

    def move(self, delta: int) -> None:
        self.select(self.index + delta)

    def select(self, index: int) -> None:
        self.index = index % len(EQUIPMENT_SLOTS)
        self.refresh()

    def set_position(self, left: float, top: float) -> None:
        self.left = left
        self.top = top
        self.bottom = self.top - self.panel_height

        for index in range(len(EQUIPMENT_SLOTS)):
            slot_left = self._slot_left()
            bottom = self._slot_bottom(index)
            center_x = slot_left + EQUIPMENT_SLOT_SIZE / 2
            center_y = bottom + EQUIPMENT_SLOT_SIZE / 2

            self.slot_icons[index].center_x = center_x
            self.slot_icons[index].center_y = center_y
            self.icons[index].center_x = center_x
            self.icons[index].center_y = center_y
            self.highlights[index] = (
                slot_left,
                slot_left + EQUIPMENT_SLOT_SIZE,
                bottom,
                bottom + EQUIPMENT_SLOT_SIZE,
            )

        self.label.x = self.left + PANEL_PADDING
        self.label.y = self.bottom + LABEL_HEIGHT / 2
        self.label.width = self.panel_width - PANEL_PADDING * 2

    def draw(self, focused: bool) -> None:
        arcade.draw_lrbt_rectangle_filled(
            self.left,
            self.left + self.panel_width,
            self.bottom,
            self.top,
            PANEL_COLOR,
        )

        for index, (slot_left, right, bottom, top) in enumerate(self.highlights):
            color = (
                SLOT_SELECTED_COLOR
                if focused and index == self.index
                else EQUIPMENT_SLOT_BG
            )
            arcade.draw_lrbt_rectangle_filled(
                slot_left,
                right,
                bottom,
                top,
                color,
            )

        self.slot_icons.draw(pixelated=True)
        self.icons.draw(pixelated=True)
        self.label.draw()

    def _slot_left(self) -> float:
        return self.left + PANEL_PADDING

    def _slot_bottom(self, row: int) -> float:
        return (
            self.top
            - PANEL_PADDING
            - (row + 1) * EQUIPMENT_SLOT_SIZE
            - row * EQUIPMENT_SLOT_GAP
        )

    def refresh(self) -> None:
        for index, sprite in enumerate(self.slot_icons):
            slot = EQUIPMENT_SLOTS[index]
            item_id = self.equipment.slots[slot]
            sprite.texture = (
                BLURRED_SLOT_TEXTURES[slot]
                if item_id is not None
                else SLOT_TEXTURES[slot]
            )

        for index, sprite in enumerate(self.icons):
            item_id = self.equipment.slots[EQUIPMENT_SLOTS[index]]
            sprite.visible = item_id is not None

            if item_id is not None:
                sprite.texture = ITEMS[item_id].texture

        slot = self.selected_slot
        item_id = self.selected_item_id
        if item_id is None:
            self.label.text = slot.replace("_", " ").title()
        else:
            self.label.text = f"{ITEMS[item_id].name} · Enter"


class InventoryPanel:
    def __init__(self, left: float, top: float, inventory: Inventory):
        self.inventory = inventory
        self.index = 0

        grid_width = COLUMNS * SLOT_SIZE + (COLUMNS - 1) * SLOT_GAP
        grid_height = ROWS * SLOT_SIZE + (ROWS - 1) * SLOT_GAP

        self.panel_width = grid_width + PANEL_PADDING * 2
        self.panel_height = grid_height + PANEL_PADDING * 2 + LABEL_HEIGHT

        self.left = left
        self.top = top
        self.bottom = self.top - self.panel_height

        self.icons = arcade.SpriteList()
        self.counts: list[arcade.Text] = []

        for index in range(len(self.inventory.slots)):
            column = index % COLUMNS
            row = index // COLUMNS
            slot_left = self._slot_left(column)
            bottom = self._slot_bottom(row)

            self.icons.append(
                arcade.Sprite(
                    ITEMS["apple"].texture,
                    scale=ICON_SCALE,
                    center_x=slot_left + SLOT_SIZE / 2,
                    center_y=bottom + SLOT_SIZE / 2,
                    pixelated=True,
                )
            )
            self.counts.append(
                arcade.Text(
                    "",
                    slot_left + SLOT_SIZE - 4,
                    bottom + 4,
                    TEXT_COLOR,
                    font_size=COUNT_SIZE,
                    font_name=FONT_NAME,
                    anchor_x="right",
                    anchor_y="bottom",
                )
            )

        self.label = arcade.Text(
            "",
            self.left + PANEL_PADDING,
            self.bottom + LABEL_HEIGHT / 2,
            TEXT_COLOR,
            font_size=LABEL_SIZE,
            font_name=FONT_NAME,
            width=self.panel_width - PANEL_PADDING * 2,
            anchor_x="left",
            anchor_y="center",
        )
        self.refresh()

    @property
    def selected(self) -> Stack | None:
        return self.inventory.slots[self.index]

    def move(self, delta: int) -> None:
        self.select(self.index + delta)

    def select(self, index: int) -> None:
        self.index = index % len(self.inventory.slots)
        self.refresh()

    def set_position(self, left: float, top: float) -> None:
        self.left = left
        self.top = top
        self.bottom = self.top - self.panel_height

        for index in range(len(self.inventory.slots)):
            column = index % COLUMNS
            row = index // COLUMNS
            slot_left = self._slot_left(column)
            bottom = self._slot_bottom(row)
            center_x = slot_left + SLOT_SIZE / 2
            center_y = bottom + SLOT_SIZE / 2

            self.icons[index].center_x = center_x
            self.icons[index].center_y = center_y
            self.counts[index].x = slot_left + SLOT_SIZE - 4
            self.counts[index].y = bottom + 4

        self.label.x = self.left + PANEL_PADDING
        self.label.y = self.bottom + LABEL_HEIGHT / 2
        self.label.width = self.panel_width - PANEL_PADDING * 2

    def draw(self, focused: bool) -> None:
        arcade.draw_lrbt_rectangle_filled(
            self.left,
            self.left + self.panel_width,
            self.bottom,
            self.top,
            PANEL_COLOR,
        )

        for index in range(len(self.inventory.slots)):
            slot_left = self._slot_left(index % COLUMNS)
            bottom = self._slot_bottom(index // COLUMNS)
            color = (
                SLOT_SELECTED_COLOR
                if focused and index == self.index
                else SLOT_COLOR
            )

            arcade.draw_lrbt_rectangle_filled(
                slot_left,
                slot_left + SLOT_SIZE,
                bottom,
                bottom + SLOT_SIZE,
                color,
            )

        for count in self.counts:
            count.draw()

        self.icons.draw(pixelated=True)
        self.label.draw()

    def _slot_left(self, column: int) -> float:
        return self.left + PANEL_PADDING + column * (SLOT_SIZE + SLOT_GAP)

    def _slot_bottom(self, row: int) -> float:
        return self.top - PANEL_PADDING - (row + 1) * SLOT_SIZE - row * SLOT_GAP

    def refresh(self) -> None:
        for index, sprite in enumerate(self.icons):
            stack = self.inventory.slots[index]
            sprite.visible = stack is not None

            if stack is None:
                self.counts[index].text = ""
                continue

            sprite.texture = ITEMS[stack.item_id].texture
            self.counts[index].text = (
                str(stack.count) if stack.count > 1 else ""
            )

        stack = self.selected
        if stack is None:
            self.label.text = "—"
            return

        item = ITEMS[stack.item_id]
        name = item.name
        if stack.count > 1:
            name = f"{name} ×{stack.count}"

        if item.item_type == "consumable":
            self.label.text = f"{name} · Enter"
        else:
            self.label.text = name


class InventoryScreen:
    def __init__(
        self,
        width: int,
        height: int,
        inventory: Inventory,
        equipment: Equipment | None = None,
    ):
        self.inventory = inventory
        self.equipment = equipment or Equipment()
        self.is_open = False
        self.focus = "inventory"

        combined_left, combined_top, equipment_width = self._layout(width, height)
        self.equipment_panel = EquipmentPanel(
            combined_left,
            combined_top,
            self.equipment,
        )
        self.inventory_panel = InventoryPanel(
            combined_left + equipment_width + PANEL_GAP,
            combined_top,
            self.inventory,
        )

    @staticmethod
    def _layout(width: int, height: int) -> tuple[float, float, float]:
        equipment_height = (
            len(EQUIPMENT_SLOTS) * EQUIPMENT_SLOT_SIZE
            + (len(EQUIPMENT_SLOTS) - 1) * EQUIPMENT_SLOT_GAP
            + PANEL_PADDING * 2
            + LABEL_HEIGHT
        )
        inventory_height = (
            ROWS * SLOT_SIZE
            + (ROWS - 1) * SLOT_GAP
            + PANEL_PADDING * 2
            + LABEL_HEIGHT
        )
        equipment_width = EQUIPMENT_SLOT_SIZE + PANEL_PADDING * 2
        inventory_width = (
            COLUMNS * SLOT_SIZE
            + (COLUMNS - 1) * SLOT_GAP
            + PANEL_PADDING * 2
        )

        combined_width = equipment_width + PANEL_GAP + inventory_width
        combined_height = max(equipment_height, inventory_height)
        combined_left = (width - combined_width) / 2
        combined_top = (height + combined_height) / 2

        return combined_left, combined_top, equipment_width

    def resize(self, width: int, height: int) -> None:
        combined_left, combined_top, equipment_width = self._layout(width, height)
        self.equipment_panel.set_position(combined_left, combined_top)
        self.inventory_panel.set_position(
            combined_left + equipment_width + PANEL_GAP,
            combined_top,
        )

    def toggle(self) -> None:
        self.is_open = not self.is_open
        if self.is_open:
            self.refresh()

    def close(self) -> None:
        self.is_open = False

    def switch_focus(self) -> None:
        self.focus = "equipment" if self.focus == "inventory" else "inventory"
        self.refresh()

    def handle_key(self, symbol: int) -> str | ConfirmResult | None:
        if symbol in (arcade.key.I, arcade.key.ESCAPE):
            return "close"

        if symbol == arcade.key.TAB:
            self.switch_focus()
            return None

        if symbol in (arcade.key.ENTER, arcade.key.SPACE):
            return self.confirm()

        if self.focus == "equipment":
            if symbol in (arcade.key.UP, arcade.key.W):
                self.move(-1)
            elif symbol in (arcade.key.DOWN, arcade.key.S):
                self.move(1)
            elif symbol == arcade.key.RIGHT:
                self.focus = "inventory"
                self.refresh()
            return None

        if symbol in (arcade.key.LEFT, arcade.key.A):
            if self.inventory_panel.index % COLUMNS == 0:
                self.focus = "equipment"
                self.refresh()
            else:
                self.move(-1)
        elif symbol in (arcade.key.RIGHT, arcade.key.D):
            self.move(1)
        elif symbol in (arcade.key.UP, arcade.key.W):
            self.move(-COLUMNS)
        elif symbol in (arcade.key.DOWN, arcade.key.S):
            self.move(COLUMNS)

        return None

    def move(self, delta: int) -> None:
        if self.focus == "equipment":
            self.equipment_panel.move(delta)
            return

        self.inventory_panel.move(delta)

    def confirm(self) -> ConfirmResult | None:
        if self.focus == "equipment":
            if self._unequip_selected():
                return ConfirmResult("unequip")
            return None

        stack = self.inventory_panel.selected
        if stack is None:
            return None

        item = ITEMS[stack.item_id]
        if item.item_type == "consumable":
            return self._consume_selected()

        if item.slot is not None:
            self._equip_selected()
            return ConfirmResult("equip")

        return None

    def draw(self) -> None:
        if not self.is_open:
            return

        self.equipment_panel.draw(self.focus == "equipment")
        self.inventory_panel.draw(self.focus == "inventory")

    def refresh(self) -> None:
        self.equipment_panel.refresh()
        self.inventory_panel.refresh()

    def _equip_selected(self) -> None:
        stack = self.inventory_panel.selected
        if stack is None:
            return

        item = ITEMS[stack.item_id]
        if item.slot is None:
            return

        removed = self.inventory.remove_at(self.inventory_panel.index, 1)
        if removed is None:
            return

        item_id, _ = removed
        previous = self.equipment.equip(item.slot, item_id)
        if previous is not None:
            self.inventory.add(previous, 1)

        self.refresh()

    def _consume_selected(self) -> ConfirmResult | None:
        stack = self.inventory_panel.selected
        if stack is None:
            return None

        item = ITEMS[stack.item_id]
        if item.item_type != "consumable" or item.heal <= 0:
            return None

        removed = self.inventory.remove_at(self.inventory_panel.index, 1)
        if removed is None:
            return None

        self.refresh()
        return ConfirmResult("consume", heal=item.heal)

    def _unequip_selected(self) -> bool:
        slot = self.equipment_panel.selected_slot
        item_id = self.equipment.unequip(slot)
        if item_id is None:
            return False

        if not self.inventory.add(item_id, 1):
            self.equipment.equip(slot, item_id)
            return False

        self.refresh()
        return True
