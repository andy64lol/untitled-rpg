import arcade

from config import *


class CameraController:
    def __init__(self, width: int, height: int):
        self.width = width
        self.height = height

        self.world = arcade.Camera2D()
        self.gui = arcade.Camera2D()
        self.world.zoom = CAMERA_PLAYER_ZOOM
        self.reset()

    def reset(self) -> None:
        self.world.position = (self.width / 2, self.height / 2)
        self.gui.position = (self.width / 2, self.height / 2)

    def follow(self, x: float, y: float) -> None:
        self.world.position = (x, y)

    def use_world(self) -> None:
        self.world.use()

    def use_gui(self) -> None:
        self.gui.use()
