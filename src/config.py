from pathlib import Path

import arcade


# Project paths
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
PLAYER_SPRITE_PATH = ASSETS_DIR / "sprites" / "player.png"

# Shared gameplay values
CAMERA_PLAYER_ZOOM = 3.0
ATTACK_EFFECT_DURATION = 0.15
ATTACK_EFFECT_SIZE = 32
FACING_OFFSETS = {
    "up": (0, 1),
    "down": (0, -1),
    "left": (-1, 0),
    "right": (1, 0),
}
KEY_TO_FACING = {
    arcade.key.W: "up",
    arcade.key.S: "down",
    arcade.key.A: "left",
    arcade.key.D: "right",
}
INTERACTION_KEYS = (arcade.key.E, arcade.key.ENTER)

# Player
PLAYER_GRID_SIZE = 32
PLAYER_GRID_OFFSET = 16
PLAYER_SPAWN_X = 400
PLAYER_SPAWN_Y = 300
PLAYER_SPEED = 160
PLAYER_MAX_HP = 100
PLAYER_DEBUG = True
PLAYER_DEBUG_INTERVAL = 0.25

# Map objects
OBJECT_LAYER_NAME = "objects"
OBJECT_COLLISION_SIZE = 32
SPAWN_OBJECT_NAME = "playerSpawn"
CHEST_PREFIX = "chest"
DOOR_OBJECT_NAME = "door"
FAKE_CHEST_PREFIX = "fakeChest"
ENEMY_SPAWN_PREFIX = "spawnEnemy"
ENEMY_PROPERTY = "enemy"
DEFAULT_ENEMY_ID = "bat"
FOUNTAIN_OBJECT_NAME = "fountain"
SWITCH_PREFIX = "switch"
PRESSURE_PLATE_PREFIX = "pressurePlate"

# Window and map progression
WINDOW_WIDTH = 1280
WINDOW_HEIGHT = 720
WINDOW_TITLE = "Untitled RPG"
MIN_WINDOW_WIDTH = 960
MIN_WINDOW_HEIGHT = 540
WINDOW_BACKGROUND = (10, 12, 20, 255)
MAP_FILES = ("map1.tmx", "map2.tmx", "map3.tmx", "map4.tmx")
TRANSITION_DURATION = 1.0

# Fountain, switch, and door messages
FOUNTAIN_HEAL = 20
FOUNTAIN_COOLDOWN = 5.0
FOUNTAIN_PROMPT = "A regenerating fountain, drink from here?"
FOUNTAIN_DRUNK = (
    "You drank from the fountain and got refreshed! (2 hearts healed)"
)
FOUNTAIN_DECLINED = "You chose not to drink."
FOUNTAIN_WAITING = "Don't drink too much water!"
SWITCH_OFF_PROMPT = "The switch is currently off, flip it?"
SWITCH_ON_PROMPT = "The switch is currently on, flip it?"
SWITCH_ON_MESSAGE = "The switch is now on."
SWITCH_OFF_MESSAGE = "The switch is now off."
PRESSURE_PLATE_PROMPT = "Step on the pressure plate?"
DOOR_PROMPT = "The door is locked. Give the key?"
FINAL_DOOR_PROMPT = "The final door needs a key. Give the key?"
DOOR_DECLINED = "You decided not to give the key."
DOOR_MISSING_KEY = "You do not have a key."
NEXT_MAP_MESSAGE = "The door opens. You go deeper into the dungeon."
WIN_TITLE = "YOU ESCAPED!"
WIN_MESSAGE = "The key turned, and the dungeon door opened."
WIN_HINT = "Press ENTER or ESC to return to the menu."

# HUD
STAT_BOX_COLOR = (72, 76, 88)
STAT_TEXT_COLOR = (235, 232, 213)
HEART_SIZE = 30
HEART_GAP = 2
HUD_LEFT = 20
HUD_TOP_OFFSET = 20
HEALTH_PER_HEART = 10
HEART_SCALE = 1
ATTACK_BOX_LEFT = 20
DEFENSE_BOX_LEFT = 78
STAT_BOX_SIZE = 52
STAT_TOP_OFFSET = 34
STAT_BOTTOM_OFFSET = 58
STAT_LABEL_OFFSET = 6
STAT_VALUE_OFFSET = 34
STAT_FONT_SIZE = 10
STAT_VALUE_FONT_SIZE = 12

# Dialogue
DIALOGUE_BOX_SCALE = 2
DIALOGUE_BOX_MARGIN = 40
DIALOGUE_TEXT_MARGIN = 40
DIALOGUE_TEXT_SIZE = 18
DIALOGUE_TEXT_SPEED = 45
DIALOGUE_FAST_SPEED_MULTIPLIER = 6
DIALOGUE_TEXT_COLOR = (235, 232, 213)

# Chest interaction
CHEST_REWARD_IDS = (
    "apple",
    "iron_sword",
    "rusty_iron_sword",
    "leather_armour",
    "titanium_sword",
)
FAKE_CHEST_MESSAGES = (
    "It's a fake chest! Nothing inside but your own greed.",
    "Empty. The chest seems to be laughing at you.",
    "You fell for it. There is nothing here but dust.",
    "This chest was never real. Your greed was, though.",
    "The lid creaks open to reveal... disappointment.",
)
FAKE_CHEST_AGAIN = "Still empty. Still greedy."
INTERACTION_DISTANCE = 40

# Inventory
COLUMNS = 4
ROWS = 3
SLOT_SIZE = 48
SLOT_GAP = 8
ICON_SCALE = 4
MAX_STACK = 16
STARTING_GOLD = 0
SELL_KEY = arcade.key.R
SELL_KEY_LABEL = "R"
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

# Menu
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
MENU_BACKGROUND = (10, 12, 20)
MENU_MUTED_TEXT = (142, 151, 164)
WIN_FONT_SIZE = 32
WIN_MESSAGE_FONT_SIZE = 14
WIN_HINT_FONT_SIZE = 11

# Items and font
BASE_ATTACK = 1
BASE_DEFENSE = 1
FONT_NAME = "Pixeloid Mono"
