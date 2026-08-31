from dataclasses import dataclass

import arcade

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
LABEL_HEIGHT = 28

PANEL_COLOR = (16, 18, 28, 240)
SLOT_COLOR = (38, 42, 58)
SLOT_SELECTED_COLOR = (96, 108, 148)
TEXT_COLOR = (235, 232, 213)
LABEL_SIZE = 14
COUNT_SIZE = 11


@dataclass
class Stack:
    item_id: str
    count: int = 1


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

        self.frames = arcade.SpriteList()
        self.icons = arcade.SpriteList()
        self.highlights: list[tuple[float, float, float, float]] = []

        for index, slot in enumerate(EQUIPMENT_SLOTS):
            left = self._slot_left()
            bottom = self._slot_bottom(index)

            self.frames.append(
                arcade.Sprite(
                    arcade.load_texture(
                        str(EQUIPMENT_SLOTS_DIR / f"{slot}.png")
                    ),
                    center_x=left + EQUIPMENT_SLOT_SIZE / 2,
                    center_y=bottom + EQUIPMENT_SLOT_SIZE / 2,
                    pixelated=True,
                )
            )
            self.icons.append(
                arcade.Sprite(
                    ITEMS["apple"].texture,
                    scale=ICON_SCALE,
                    center_x=left + EQUIPMENT_SLOT_SIZE / 2,
                    center_y=bottom + EQUIPMENT_SLOT_SIZE / 2,
                    pixelated=True,
                )
            )
            self.highlights.append(
                (
                    left,
                    left + EQUIPMENT_SLOT_SIZE,
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

    def draw(self, focused: bool) -> None:
        arcade.draw_lrbt_rectangle_filled(
            self.left,
            self.left + self.panel_width,
            self.bottom,
            self.top,
            PANEL_COLOR,
        )

        for index, (left, right, bottom, top) in enumerate(self.highlights):
            if focused and index == self.index:
                arcade.draw_lrbt_rectangle_filled(
                    left,
                    right,
                    bottom,
                    top,
                    SLOT_SELECTED_COLOR,
                )

        self.frames.draw(pixelated=True)
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
        for index, sprite in enumerate(self.icons):
            item_id = self.equipment.slots[EQUIPMENT_SLOTS[index]]
            sprite.visible = item_id is not None

            if item_id is not None:
                sprite.texture = ITEMS[item_id].texture

        item_id = self.selected_item_id
        if item_id is None:
            self.label.text = EQUIPMENT_SLOTS[self.index].replace("_", " ").title()
        else:
            self.label.text = ITEMS[item_id].name


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
            self.label.text = "Empty"
        else:
            self.label.text = f"{ITEMS[stack.item_id].name} x{stack.count}"


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

    def toggle(self) -> None:
        self.is_open = not self.is_open
        if self.is_open:
            self.refresh()

    def close(self) -> None:
        self.is_open = False

    def switch_focus(self) -> None:
        self.focus = "equipment" if self.focus == "inventory" else "inventory"
        self.refresh()

    def move(self, delta: int) -> None:
        if self.focus == "equipment":
            self.equipment_panel.move(delta)
            return

        self.inventory_panel.move(delta)

    def confirm(self) -> None:
        if self.focus == "equipment":
            self._unequip_selected()
            return

        self._equip_selected()

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

    def _unequip_selected(self) -> None:
        slot = self.equipment_panel.selected_slot
        item_id = self.equipment.unequip(slot)
        if item_id is None:
            return

        if not self.inventory.add(item_id, 1):
            self.equipment.equip(slot, item_id)

        self.refresh()
