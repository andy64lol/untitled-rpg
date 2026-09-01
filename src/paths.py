from pathlib import Path


SRC_DIR = Path(__file__).resolve().parent
ASSETS_DIR = SRC_DIR / "assets"
MAPS_DIR = SRC_DIR / "maps"

SAVE_PATH = SRC_DIR / "saves" / "slot1.urpgs"

ITEMS_DIR = ASSETS_DIR / "items"
ITEMS_JSON_PATH = SRC_DIR / "items.json"
ENEMIES_DIR = ASSETS_DIR / "sprites" / "enemy"
ENEMY_JSON_PATH = SRC_DIR / "enemy.json"
EQUIPMENT_SLOTS_DIR = ASSETS_DIR / "UI" / "player_equipment_slots"

FONT_PATH = ASSETS_DIR / "UI" / "font" / "PixeloidMono-nAOpP.ttf"
TITLE_PATH = ASSETS_DIR / "UI" / "menu" / "game_title.png"
DIALOGUE_BOX_PATH = ASSETS_DIR / "UI" / "dialogue" / "dialogue_box.png"
HEART_PATH = ASSETS_DIR / "UI" / "heart.png"
BROKEN_HEART_PATH = ASSETS_DIR / "UI" / "broken_heart.png"
