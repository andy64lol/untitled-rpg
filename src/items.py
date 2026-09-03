import json
from dataclasses import dataclass

import arcade

from config import *


@dataclass(frozen=True)
class Item:
    id: str
    name: str
    texture: arcade.Texture
    item_type: str
    slot: str | None
    stackable: bool
    damage: int = 0
    defense: int = 0
    heal: int = 0


def get_combat_stats(equipment) -> tuple[int, int]:
    attack = BASE_ATTACK
    defense = BASE_DEFENSE

    for item_id in equipment.slots.values():
        if item_id is None:
            continue

        item = ITEMS[item_id]
        attack += item.damage
        defense += item.defense

    return attack, defense


def load_items() -> dict[str, Item]:
    with ITEMS_JSON_PATH.open(encoding="utf-8") as handle:
        data = json.load(handle)

    items: dict[str, Item] = {}
    for item_id, entry in data.items():
        items[item_id] = Item(
            id=item_id,
            name=entry["name"],
            texture=arcade.load_texture(str(ITEMS_DIR / entry["texture"])),
            item_type=entry["type"],
            slot=entry.get("slot"),
            stackable=entry.get("stackable", False),
            damage=entry.get("damage", 0),
            defense=entry.get("defense", 0),
            heal=entry.get("heal", 0),
        )

    return items


ITEMS = load_items()
