from pathlib import Path
from typing import Any

import arcade

from config import *


def snap_to_tile_center(value: float, tile_size: float) -> float:
    """Return the center of the closest tile for a map object coordinate."""
    return (
        round((value - tile_size / 2) / tile_size) * tile_size
        + tile_size / 2
    )


class GameMap:
    def __init__(self, map_path: Path):
        self.path = map_path
        self.tilemap = arcade.load_tilemap(str(map_path))
        self.scene = arcade.Scene.from_tilemap(self.tilemap)
        self.world_width = self.tilemap.width * self.tilemap.tile_width
        self.world_height = self.tilemap.height * self.tilemap.tile_height

        self.spikes = self._layer("spikes")
        self.collision = self._layer("collision")
        map_objects = self.tilemap.object_lists.get(OBJECT_LAYER_NAME, [])

        self.fountains = self._objects_named(
            map_objects, FOUNTAIN_OBJECT_NAME
        )
        self.switches = self._objects_starting_with(
            map_objects, SWITCH_PREFIX
        )
        self.pressure_plates = self._objects_starting_with(
            map_objects, PRESSURE_PLATE_PREFIX
        )
        self.chests = self._objects_starting_with(map_objects, CHEST_PREFIX)
        self.fake_chests = self._objects_starting_with(
            map_objects, FAKE_CHEST_PREFIX
        )
        self.door = self._objects_named(map_objects, DOOR_OBJECT_NAME)
        self.spawn = self._spawn_position(map_objects)
        self.enemy_spawns = self._enemy_spawn_points(
            self._objects_starting_with(map_objects, ENEMY_SPAWN_PREFIX)
        )

        self.fountain_colliders = self._make_colliders(self.fountains)
        self.switch_colliders = self._make_colliders(self.switches)
        self.pressure_plate_colliders = self._make_colliders(
            self.pressure_plates
        )
        self.chest_colliders = self._make_colliders(self.chests)
        self.fake_chest_colliders = self._make_colliders(self.fake_chests)
        self.door_colliders = self._make_colliders(self.door)
        self.switch_states = [False] * len(self.switches)
        self.pressure_plate_states = [False] * len(self.pressure_plates)

        self.collision.extend(self.fountain_colliders)
        self.collision.extend(self.switch_colliders)
        self.collision.extend(self.pressure_plate_colliders)
        self.collision.extend(self.chest_colliders)
        self.collision.extend(self.fake_chest_colliders)
        self.collision.extend(self.door_colliders)

    def _layer(self, name: str) -> arcade.SpriteList[arcade.Sprite]:
        try:
            return self.scene[name]
        except KeyError:
            return arcade.SpriteList()

    def _spawn_position(self, objects: list[Any]) -> tuple[float, float]:
        spawns = self._objects_named(objects, SPAWN_OBJECT_NAME)
        if spawns:
            return self.object_position(spawns[0])

        return self.world_width / 2, self.world_height / 2

    def _enemy_spawn_points(
        self,
        objects: list[Any],
    ) -> list[tuple[float, float, str]]:
        spawn_points = []
        for map_object in objects:
            x, y = self.object_position(map_object)
            properties = map_object.properties or {}
            enemy_id = properties.get(
                ENEMY_PROPERTY, DEFAULT_ENEMY_ID
            )
            spawn_points.append((x, y, enemy_id))

        return spawn_points

    @staticmethod
    def _objects_named(objects: list[Any], name: str) -> list[Any]:
        return [obj for obj in objects if obj.name == name]

    @staticmethod
    def _objects_starting_with(objects: list[Any], prefix: str) -> list[Any]:
        return sorted(
            (obj for obj in objects if obj.name.startswith(prefix)),
            key=lambda obj: obj.name,
        )

    def object_position(self, map_object: Any) -> tuple[float, float]:
        shape = map_object.shape
        return (
            snap_to_tile_center(float(shape[0]), self.tilemap.tile_width),
            snap_to_tile_center(float(shape[1]), self.tilemap.tile_height),
        )

    def _make_colliders(self, objects: list[Any]) -> arcade.SpriteList[arcade.Sprite]:
        colliders = arcade.SpriteList()
        for map_object in objects:
            center_x, center_y = self.object_position(map_object)
            collider = arcade.SpriteSolidColor(
                OBJECT_COLLISION_SIZE,
                OBJECT_COLLISION_SIZE,
                color=(0, 0, 0, 0),
            )
            collider.center_x = center_x
            collider.center_y = center_y
            colliders.append(collider)
        return colliders

    def _index_in_collision(
        self,
        collision: list[arcade.Sprite],
        colliders: arcade.SpriteList[arcade.Sprite],
    ) -> int | None:
        for index, collider in enumerate(colliders):
            if collider in collision:
                return index
        return None

    def switch_index_in_collision(
        self, collision: list[arcade.Sprite]
    ) -> int | None:
        return self._index_in_collision(collision, self.switch_colliders)

    def pressure_plate_index_in_collision(
        self, collision: list[arcade.Sprite]
    ) -> int | None:
        return self._index_in_collision(
            collision, self.pressure_plate_colliders
        )

    def _index_in_front(
        self,
        objects: list[Any],
        player: arcade.Sprite,
        facing: str,
    ) -> int | None:
        offset_x, offset_y = FACING_OFFSETS[facing]
        target_x = player.center_x + offset_x * player.width
        target_y = player.center_y + offset_y * player.height

        closest_index = None
        closest_distance = float("inf")
        for index, map_object in enumerate(objects):
            object_x, object_y = self.object_position(map_object)
            distance = (
                (object_x - target_x) ** 2 + (object_y - target_y) ** 2
            ) ** 0.5
            if distance <= INTERACTION_DISTANCE and distance < closest_distance:
                closest_index = index
                closest_distance = distance

        return closest_index

    def switch_index_in_front(
        self, player: arcade.Sprite, facing: str
    ) -> int | None:
        return self._index_in_front(self.switches, player, facing)

    def pressure_plate_index_in_front(
        self, player: arcade.Sprite, facing: str
    ) -> int | None:
        return self._index_in_front(self.pressure_plates, player, facing)

    def toggle_switch(self, index: int) -> bool:
        self.switch_states[index] = not self.switch_states[index]
        return self.switch_states[index]

    def activate_pressure_plate(self, index: int) -> None:
        self.pressure_plate_states[index] = True

    def save_state(self) -> dict[str, list[bool]]:
        return {
            "switches": list(self.switch_states),
            "pressure_plates": list(self.pressure_plate_states),
        }

    def load_state(self, state: Any) -> None:
        if not isinstance(state, dict):
            return

        saved_switches = state.get("switches", [])
        if isinstance(saved_switches, list):
            self.switch_states = [
                bool(saved_switches[index])
                if index < len(saved_switches)
                else False
                for index in range(len(self.switches))
            ]

        saved_plates = state.get("pressure_plates", [])
        if isinstance(saved_plates, list):
            self.pressure_plate_states = [
                bool(saved_plates[index])
                if index < len(saved_plates)
                else False
                for index in range(len(self.pressure_plates))
            ]

    def draw(self, pixelated: bool = False) -> None:
        self.scene.draw(pixelated=pixelated)

    def door_is_in_front(
        self,
        player: arcade.Sprite,
        facing: str,
    ) -> bool:
        offset_x, offset_y = {
            "up": (0, 1),
            "down": (0, -1),
            "left": (-1, 0),
            "right": (1, 0),
        }[facing]
        target_x = player.center_x + offset_x * player.width
        target_y = player.center_y + offset_y * player.height

        return any(
            (
                (self.object_position(door)[0] - target_x) ** 2
                + (self.object_position(door)[1] - target_y) ** 2
            )
            ** 0.5
            <= 40
            for door in self.door
        )

    def door_is_in_collision(self, collision: list[arcade.Sprite]) -> bool:
        return any(door in collision for door in self.door_colliders)

    def unlock_door(self) -> None:
        for door in list(self.door_colliders):
            if door in self.collision:
                self.collision.remove(door)

    def lock_door(self) -> None:
        for door in self.door_colliders:
            if door not in self.collision:
                self.collision.append(door)