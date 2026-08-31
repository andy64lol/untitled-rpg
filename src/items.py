import json
from dataclasses import dataclass

import arcade

from paths import ITEMS_DIR, ITEMS_JSON_PATH


@dataclass(frozen=True)
class Item:
    id: str
    name: str
    texture: arcade.Texture
    item_type: str
    slot: str | None
    stackable: bool


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
        )

    return items


ITEMS = load_items()
