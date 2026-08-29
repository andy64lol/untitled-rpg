from pathlib import Path
import arcade


playerPath = Path("src/assets/sprites/player.png")

spriteSheet = arcade.load_spritesheet(playerPath)

frames = spriteSheet.get_texture_grid(
    size=(32, 32),
    columns=8,
    count=32,
)

playerSprite = arcade.Sprite(
    frames[0],
    center_x=400,
    center_y=300,
)

playerList = arcade.SpriteList()
playerList.append(playerSprite)


def draw():
    playerList.draw()
