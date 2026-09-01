import json
from dataclasses import dataclass
from typing import Any

import arcade

from map import snap_to_tile_center
from paths import ENEMIES_DIR, ENEMY_JSON_PATH

@dataclass(frozen=True)
class EnemyType:
    id: str
    name: str
    frames: tuple[arcade.Texture, ...]
    frame_duration: float
    speed: float
    damage: int


def load_enemies() -> dict[str, EnemyType]:
    with ENEMY_JSON_PATH.open(encoding="utf-8") as handle:
        data = json.load(handle)

    enemies: dict[str, EnemyType] = {}
    for enemy_id, entry in data.items():
        sheet = arcade.load_spritesheet(str(ENEMIES_DIR / entry["texture"]))
        frames = sheet.get_texture_grid(
            size=tuple(entry["frame_size"]),
            columns=entry["frame_columns"],
            count=entry["frame_count"],
        )
        enemies[enemy_id] = EnemyType(
            id=enemy_id,
            name=entry["name"],
            frames=tuple(frames),
            frame_duration=entry["frame_duration"],
            speed=entry["speed"],
            damage=entry.get("damage", 0),
        )

    return enemies


ENEMIES = load_enemies()


class Enemy(arcade.Sprite):
    def __init__(self, kind: EnemyType, center_x: float, center_y: float):
        super().__init__(
            kind.frames[0],
            center_x=center_x,
            center_y=center_y,
            pixelated=True,
        )
        self.kind = kind
        self.frame_index = 0
        self.frame_timer = 0.0
        self.target_x = center_x
        self.target_y = center_y
        self.moving = False

    def update_animation(self, delta_time: float = 1 / 60, *args) -> None:
        self.frame_timer += delta_time
        if self.frame_timer < self.kind.frame_duration:
            return

        self.frame_timer %= self.kind.frame_duration
        self.frame_index = (self.frame_index + 1) % len(self.kind.frames)
        self.texture = self.kind.frames[self.frame_index]

    def step_to(self, center_x: float, center_y: float) -> None:
        self.target_x = center_x
        self.target_y = center_y
        self.moving = True

    def update_movement(self, delta_time: float) -> None:
        if not self.moving:
            return

        distance_x = self.target_x - self.center_x
        distance_y = self.target_y - self.center_y
        distance = (distance_x**2 + distance_y**2) ** 0.5
        move_distance = self.kind.speed * delta_time

        if distance <= move_distance:
            self.center_x = self.target_x
            self.center_y = self.target_y
            self.moving = False
            return

        self.center_x += distance_x / distance * move_distance
        self.center_y += distance_y / distance * move_distance


class EnemyManager:
    def __init__(
        self,
        collision: arcade.SpriteList[arcade.Sprite],
        spawn_points: list[tuple[float, float, str]] = (),
        tile_width: float = 32,
        tile_height: float = 32,
    ):
        self.collision = collision
        self.tile_width = tile_width
        self.tile_height = tile_height
        self.enemies = arcade.SpriteList[Enemy]()
        self.spawn_points = list(spawn_points)
        self.armed: set[int] = set()
        self.spawned: set[int] = set()

    @property
    def alive_count(self) -> int:
        return len(self.enemies)

    def reset(self) -> None:
        self.enemies.clear()
        self.armed.clear()
        self.spawned.clear()

    def spawn(self, enemy_id: str, center_x: float, center_y: float) -> None:
        kind = ENEMIES.get(enemy_id)
        if kind is None:
            return

        self.enemies.append(Enemy(kind, center_x, center_y))

    def update(self, delta_time: float) -> None:
        for enemy in self.enemies:
            enemy.update_movement(delta_time)

        self.enemies.update_animation(delta_time)

    def update_triggers(self, player: arcade.Sprite) -> None:
        """Spawn an enemy on a trigger tile once the player steps off it."""
        for index, (spawn_x, spawn_y, enemy_id) in enumerate(self.spawn_points):
            if index in self.spawned:
                continue

            if self._tile_of(player.targetX, player.targetY) == (
                spawn_x,
                spawn_y,
            ):
                self.armed.add(index)
            elif index in self.armed:
                self.armed.discard(index)
                self.spawned.add(index)
                self.spawn(enemy_id, spawn_x, spawn_y)

    def step(self, player: arcade.Sprite) -> None:
        """Move every enemy one tile, the player just finished moving."""
        for enemy in self.enemies:
            if enemy.moving:
                continue

            for offset_x, offset_y in self._steps_toward(enemy, player):
                target_x = enemy.center_x + offset_x
                target_y = enemy.center_y + offset_y
                if not self._is_blocked(enemy, target_x, target_y):
                    enemy.step_to(target_x, target_y)
                    break

    def contact_damage(self, player: arcade.Sprite) -> int:
        return max(
            (
                enemy.kind.damage
                for enemy in arcade.check_for_collision_with_list(
                    player, self.enemies
                )
            ),
            default=0,
        )

    def draw(self, pixelated: bool = True) -> None:
        self.enemies.draw(pixelated=pixelated)

    def save_state(self) -> dict[str, Any]:
        return {
            "enemies": [
                {"type": enemy.kind.id, "x": enemy.center_x, "y": enemy.center_y}
                for enemy in self.enemies
            ],
            "spawned": sorted(self.spawned),
        }

    def load_state(self, state: dict[str, Any]) -> None:
        self.enemies.clear()
        self.armed.clear()

        for entry in state.get("enemies", []):
            enemy_id = entry.get("type")
            if enemy_id in ENEMIES:
                self.spawn(enemy_id, float(entry["x"]), float(entry["y"]))

        self.spawned = {
            int(index)
            for index in state.get("spawned", [])
            if isinstance(index, int) and 0 <= index < len(self.spawn_points)
        }

    def _tile_of(self, x: float, y: float) -> tuple[float, float]:
        return (
            snap_to_tile_center(x, self.tile_width),
            snap_to_tile_center(y, self.tile_height),
        )

    def _steps_toward(
        self,
        enemy: Enemy,
        player: arcade.Sprite,
    ) -> list[tuple[float, float]]:
        offset_x = player.center_x - enemy.center_x
        offset_y = player.center_y - enemy.center_y
        step_x = self.tile_width if offset_x > 0 else -self.tile_width
        step_y = self.tile_height if offset_y > 0 else -self.tile_height

        moves: list[tuple[float, float]] = []
        if abs(offset_x) >= abs(offset_y):
            if offset_x:
                moves.append((step_x, 0))
            if offset_y:
                moves.append((0, step_y))
        else:
            if offset_y:
                moves.append((0, step_y))
            if offset_x:
                moves.append((step_x, 0))

        return moves

    def _is_blocked(self, enemy: Enemy, x: float, y: float) -> bool:
        old_x = enemy.center_x
        old_y = enemy.center_y

        enemy.center_x = x
        enemy.center_y = y
        blocked = bool(
            arcade.check_for_collision_with_list(enemy, self.collision)
        )

        enemy.center_x = old_x
        enemy.center_y = old_y

        return blocked
