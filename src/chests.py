import random
from typing import Any

import arcade

from inventory import Inventory
from items import ITEMS
from map import snap_to_tile_center

CHEST_REWARD_IDS = (
    "apple",
    "iron_sword",
    "rusty_iron_sword",
    "leather_armour",
    "titanium_sword",
)
INTERACTION_DISTANCE = 40


class ChestManager:
    def __init__(
        self,
        chest_objects: list[Any],
        tile_width: float = 32,
        tile_height: float = 32,
    ):
        self.chest_objects = chest_objects
        self.tile_width = tile_width
        self.tile_height = tile_height
        self.opened: set[int] = set()

    @property
    def unopened_count(self) -> int:
        return len(self.chest_objects) - len(self.opened)

    def reset(self) -> None:
        self.opened.clear()

    def find_in_front(self, player: arcade.Sprite, facing: str) -> int | None:
        offset_x, offset_y = {
            "up": (0, 1),
            "down": (0, -1),
            "left": (-1, 0),
            "right": (1, 0),
        }[facing]
        target_x = player.center_x + offset_x * player.width
        target_y = player.center_y + offset_y * player.height

        closest_index: int | None = None
        closest_distance = float("inf")
        for index, chest in enumerate(self.chest_objects):
            chest_x, chest_y = self._object_position(chest)
            distance = (
                (chest_x - target_x) ** 2
                + (chest_y - target_y) ** 2
            ) ** 0.5
            if distance <= INTERACTION_DISTANCE and distance < closest_distance:
                closest_index = index
                closest_distance = distance

        return closest_index

    def _object_position(self, chest: Any) -> tuple[float, float]:
        chest_x, chest_y = chest.shape
        return (
            snap_to_tile_center(float(chest_x), self.tile_width),
            snap_to_tile_center(float(chest_y), self.tile_height),
        )

    def find_in_collision(
        self,
        collision: list[arcade.Sprite],
        chest_colliders: arcade.SpriteList[arcade.Sprite],
    ) -> int | None:
        for index, collider in enumerate(chest_colliders):
            if collider in collision:
                return index
        return None

    def interact(self, index: int, inventory: Inventory) -> str:
        if index in self.opened:
            return "This chest has already been opened."

        item_id = "key" if self.unopened_count == 1 else random.choice(
            CHEST_REWARD_IDS
        )
        if not inventory.add(item_id):
            return "Your inventory is full. Make room before opening this chest."

        self.opened.add(index)
        if item_id == "key":
            return "The last chest contained the key to the final door!"

        return f"The chest contained {ITEMS[item_id].name}."

    def save_state(self) -> dict[str, Any]:
        return {"opened": sorted(self.opened)}

    def load_state(self, state: dict[str, Any]) -> None:
        opened = state.get("opened", [])
        self.opened = {
            int(index)
            for index in opened
            if isinstance(index, int) and 0 <= index < len(self.chest_objects)
        }