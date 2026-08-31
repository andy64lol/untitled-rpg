import arcade

from config import *


class CameraController:
    def __init__(self, width: int, height: int):
        self.width = width
        self.height = height

        self.world = arcade.Camera2D()
        self.gui = arcade.Camera2D()
        self.world.zoom = CAMERA_PLAYER_ZOOM
        self._player_x = width / 2
        self._player_y = height / 2
        self._sync_cameras()

    def _sync_cameras(self) -> None:
        self.world.match_window()
        self.gui.match_window(position=True)
        self.world.position = (self._player_x, self._player_y)

    def reset(self) -> None:
        self._sync_cameras()

    def resize(self, width: int, height: int) -> None:
        self.width = width
        self.height = height
        self._sync_cameras()

    def follow(self, x: float, y: float) -> None:
        self._player_x = x
        self._player_y = y
        self.world.position = (x, y)

    def use_world(self) -> None:
        self.world.use()

    def use_gui(self) -> None:
        self.gui.use()
