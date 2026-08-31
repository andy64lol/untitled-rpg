import arcade

from config import *


class CameraController:
    def __init__(self, width: int, height: int):
        self.width = width
        self.height = height
        self.world_bounds: tuple[float, float, float, float] | None = None

        self.world = arcade.Camera2D()
        self.gui = arcade.Camera2D()
        self.world.zoom = CAMERA_PLAYER_ZOOM
        self._player_x = width / 2
        self._player_y = height / 2
        self._sync_cameras()

    def _sync_cameras(self) -> None:
        self.world.match_window()
        self.gui.match_window(position=True)
        self.world.position = self._clamped_position(
            self._player_x,
            self._player_y,
        )

    def reset(self) -> None:
        self._sync_cameras()

    def resize(self, width: int, height: int) -> None:
        self.width = max(1, width)
        self.height = max(1, height)
        self._sync_cameras()

    def set_world_bounds(
        self,
        left: float,
        right: float,
        bottom: float,
        top: float,
    ) -> None:
        self.world_bounds = (left, right, bottom, top)
        self.follow(self._player_x, self._player_y)

    def follow(self, x: float, y: float) -> None:
        self._player_x = x
        self._player_y = y
        self.world.position = self._clamped_position(x, y)

    def _clamped_position(self, x: float, y: float) -> tuple[float, float]:
        if self.world_bounds is None:
            return x, y

        left, right, bottom, top = self.world_bounds
        half_view_width = self.width / (2 * self.world.zoom)
        half_view_height = self.height / (2 * self.world.zoom)

        if right - left <= half_view_width * 2:
            camera_x = (left + right) / 2
        else:
            camera_x = min(max(x, left + half_view_width), right - half_view_width)

        if top - bottom <= half_view_height * 2:
            camera_y = (bottom + top) / 2
        else:
            camera_y = min(max(y, bottom + half_view_height), top - half_view_height)

        return camera_x, camera_y

    def use_world(self) -> None:
        self.world.use()

    def use_gui(self) -> None:
        self.gui.use()
