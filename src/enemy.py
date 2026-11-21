import json
import math
from collections.abc import Sequence
from dataclasses import dataclass
from typing import Any

import arcade
from PIL import Image

from config import *
from map import snap_to_tile_center
from sprites import Player


@dataclass(frozen=True)
class MobAnimation:
    frames: tuple[int, ...]
    looped: bool = True


@dataclass(frozen=True)
class EnemyType:
    id: str
    name: str
    frames: tuple[arcade.Texture, ...]
    frame_duration: float
    speed: float
    damage: int
    walking: MobAnimation
    defeat: MobAnimation


def _load_stats() -> dict[str, dict[str, Any]]:
    if not ENEMY_JSON_PATH.exists():
        return {}
    with ENEMY_JSON_PATH.open(encoding="utf-8") as handle:
        data = json.load(handle)
    return {
        entry.get("id", enemy_id): entry
        for enemy_id, entry in data.items()
    }


def _animation(
    spec: Any, frame_count: int, default: tuple[int, ...] | None = None
) -> MobAnimation:
    if isinstance(spec, dict):
        frames = spec.get("animation", "default")
        looped = bool(spec.get("looped", True))
    else:
        frames = spec
        looped = True
    if frames == "default" or frames is None:
        indices = default if default is not None else tuple(range(frame_count))
    else:
        indices = tuple(int(index) for index in frames)
    return MobAnimation(indices, looped)


def load_enemies() -> dict[str, EnemyType]:
    stats = _load_stats()
    enemies: dict[str, EnemyType] = {}

    for path in sorted(ENEMIES_DIR.glob(f"*{MOB_DATA_EXTENSION}")):
        with path.open(encoding="utf-8") as handle:
            data = json.load(handle)

        mob_id = data.get("mob_id") or path.stem
        texture_name = data.get("textures") or f"{mob_id}.png"
        frame_data = data.get("frames", {})
        frame_size = tuple(
            frame_data.get("frame_size", MOB_DEFAULT_FRAME_SIZE)
        )
        columns = int(
            frame_data.get("frame_columns", MOB_DEFAULT_FRAME_COLUMNS)
        )

        with Image.open(ENEMIES_DIR / texture_name) as image:
            rows = image.height // frame_size[1]
        frame_count = columns * rows

        sheet = arcade.load_spritesheet(str(ENEMIES_DIR / texture_name))
        frames = tuple(
            sheet.get_texture_grid(
                size=frame_size, columns=columns, count=frame_count
            )
        )

        animations = data.get("animations", {})
        walking = _animation(animations.get("walking", "default"), frame_count)
        defeat = _animation(
            animations.get("defeat", "default"),
            frame_count,
            default=(0,),
        )

        entry = stats.get(mob_id, {})
        enemies[mob_id] = EnemyType(
            id=mob_id,
            name=data.get("name", entry.get("name", mob_id.replace("_", " ").title())),
            frames=frames,
            frame_duration=float(
                frame_data.get(
                    "frame_duration", MOB_DEFAULT_FRAME_DURATION
                )
            ),
            speed=float(
                data.get("speed", entry.get("speed", MOB_DEFAULT_SPEED))
            ),
            damage=int(
                data.get("damage", entry.get("damage", MOB_DEFAULT_DAMAGE))
            ),
            walking=walking,
            defeat=defeat,
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
        self.swing_phase = 0.0
        self.flip_timer = 0.0
        self.flip_from = 1.0
        self.flip_to = 1.0
        self.facing_x = 1
        self.dying = False
        self.death_timer = 0.0

    def die(self) -> None:
        if self.dying:
            return
        self.dying = True
        self.death_timer = ENEMY_DEATH_DURATION
        self.moving = False
        self.flip_timer = 0.0
        self.scale_x = 1.0
        self.scale_y = 1.0
        self.angle = 0.0

    def update_death_animation(self, delta_time: float) -> bool:
        if not self.dying:
            return False
        self.death_timer = max(0.0, self.death_timer - delta_time)
        progress = 1.0 - self.death_timer / ENEMY_DEATH_DURATION
        frames = self.kind.defeat.frames
        if frames:
            index = min(
                len(frames) - 1,
                int(progress * len(frames)),
            )
            frame_number = frames[index]
            if 0 <= frame_number < len(self.kind.frames):
                self.texture = self.kind.frames[frame_number]
        self.scale_x = 1.0 + 0.25 * math.sin(math.pi * progress)
        self.scale_y = max(0.05, 1.0 - 0.9 * progress)
        self.angle = 80.0 * progress * (1.0 if self.facing_x > 0 else -1.0)
        self.alpha = int(255 * (1.0 - progress))
        return self.death_timer <= 0.0

    def _update_motion_animation(self, delta_time: float) -> None:
        swing_target = 0.0
        if self.moving:
            swing_sign = -1.0 if self.facing_x < 0 else 1.0
            swing_target = (
                SWING_ANGLE_DEGREES
                * swing_sign
                * (0.75 + 0.25 * math.sin(self.swing_phase))
            )
            self.swing_phase += delta_time * SWING_SPEED
        ease = min(1.0, delta_time * SWING_EASE)
        self.angle += (swing_target - self.angle) * ease
        if abs(self.angle) < 0.01:
            self.angle = 0.0

        if self.flip_timer > 0.0:
            self.flip_timer = max(0.0, self.flip_timer - delta_time)
            progress = 1.0 - self.flip_timer / FLIP_DURATION
            self.scale_x = self.flip_from + (
                self.flip_to - self.flip_from
            ) * progress
            if self.flip_timer == 0.0:
                self.scale_x = self.flip_to

    def update_animation(self, delta_time: float = 1 / 60, *args) -> None:
        if self.dying:
            return
        self.frame_timer += delta_time
        if self.frame_timer < self.kind.frame_duration:
            return

        self.frame_timer %= self.kind.frame_duration
        frames = self.kind.walking.frames
        if not frames:
            return
        self.frame_index += 1
        if self.kind.walking.looped:
            self.frame_index %= len(frames)
        else:
            self.frame_index = min(self.frame_index, len(frames) - 1)
        frame_number = frames[self.frame_index]
        if 0 <= frame_number < len(self.kind.frames):
            self.texture = self.kind.frames[frame_number]

    def step_to(self, center_x: float, center_y: float) -> None:
        self.target_x = center_x
        self.target_y = center_y
        self.moving = True
        direction_x = center_x - self.center_x
        if direction_x != 0:
            facing = 1 if direction_x > 0 else -1
            if facing != self.facing_x:
                self.flip_from = self.scale_x
                self.flip_to = float(facing)
                self.flip_timer = FLIP_DURATION
                self.facing_x = facing
                self.swing_phase = 0.0

    def update_movement(self, delta_time: float) -> None:
        if self.dying:
            return
        self._update_motion_animation(delta_time)
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
        spawn_points: Sequence[tuple[float, float, str]] = (),
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
        return sum(1 for enemy in self.enemies if not enemy.dying)

    def reset(self) -> None:
        self.enemies.clear()
        self.armed.clear()
        self.spawned.clear()

    def spawn(self, enemy_id: str, center_x: float, center_y: float) -> None:
        kind = ENEMIES.get(enemy_id) or ENEMIES.get(DEFAULT_ENEMY_ID)
        if kind is None:
            return

        self.enemies.append(Enemy(kind, center_x, center_y))

    def spawn_near_player(self, player: Player) -> bool:
        """Spawn the default bat on the closest available nearby tile."""
        kind = ENEMIES.get("bat")
        if kind is None:
            return False

        tile_offsets = (
            (-2, 0),
            (2, 0),
            (0, -2),
            (0, 2),
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
        )
        for offset_x, offset_y in tile_offsets:
            center_x = player.center_x + offset_x * self.tile_width
            center_y = player.center_y + offset_y * self.tile_height
            enemy = Enemy(kind, center_x, center_y)
            if self._is_blocked(enemy, center_x, center_y):
                continue
            if player.collides_with(enemy):
                continue

            self.enemies.append(enemy)
            return True

        return False

    def attack_at(self, center_x: float, center_y: float) -> bool:
        """Start the death animation for the first enemy in the attack tile."""
        for enemy in self.enemies:
            if enemy.dying:
                continue
            if (
                abs(enemy.center_x - center_x) <= self.tile_width
                and abs(enemy.center_y - center_y) <= self.tile_height
            ):
                enemy.die()
                return True
        return False

    def update(self, delta_time: float) -> None:
        for enemy in list(self.enemies):
            if enemy.dying:
                if enemy.update_death_animation(delta_time):
                    self.enemies.remove(enemy)
                continue
            enemy.update_movement(delta_time)

        self.enemies.update_animation(delta_time)

    def update_triggers(self, player: Player) -> None:
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

    def step(self, player: Player) -> None:
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

    def contact_damage(self, player: Player) -> int:
        return max(
            (
                enemy.kind.damage
                for enemy in player.collision_with(self.enemies)
                if isinstance(enemy, Enemy) and not enemy.dying
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
                if not enemy.dying
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
        player: Player,
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
        old_angle = enemy.angle
        old_scale_x = enemy.scale_x
        old_scale_y = enemy.scale_y

        enemy.angle = 0.0
        enemy.scale_x = 1.0
        enemy.scale_y = 1.0
        enemy.center_x = x
        enemy.center_y = y
        blocked = bool(
            arcade.check_for_collision_with_list(enemy, self.collision)
        )

        enemy.center_x = old_x
        enemy.center_y = old_y
        enemy.angle = old_angle
        enemy.scale_x = old_scale_x
        enemy.scale_y = old_scale_y

        return blocked
