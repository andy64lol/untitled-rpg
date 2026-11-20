import math
from typing import Any

import arcade

from config import *

spriteSheet = arcade.load_spritesheet(PLAYER_SPRITE_PATH)

frames = spriteSheet.get_texture_grid(
    size=(PLAYER_GRID_SIZE, PLAYER_GRID_SIZE),
    columns=8,
    count=32,
)


class Player(arcade.Sprite):
    def __init__(self, tilemap, collisionList, fountainList):
        self.gridSize = PLAYER_GRID_SIZE
        self.gridOffset = PLAYER_GRID_OFFSET

        spawnX = PLAYER_SPAWN_X
        spawnY = PLAYER_SPAWN_Y

        for objectList in tilemap.object_lists.values():
            for obj in objectList:
                if obj.name == "playerSpawn":
                    spawnX = obj.shape[0]
                    spawnY = obj.shape[1]
                    break

        spawnX = (
            round((spawnX - self.gridOffset) / self.gridSize)
            * self.gridSize
            + self.gridOffset
        )

        spawnY = (
            round((spawnY - self.gridOffset) / self.gridSize)
            * self.gridSize
            + self.gridOffset
        )

        super().__init__(
            frames[0],
            center_x=spawnX,
            center_y=spawnY,
            pixelated=True,
        )

        self.spawnX = spawnX
        self.spawnY = spawnY

        self.speed = PLAYER_SPEED
        self.maxHp = PLAYER_MAX_HP
        self.hp = self.maxHp
        self.collisionList = collisionList
        self.fountainList = fountainList
        self.facing = "right"
        self.direction = "right"
        self.dead = False
        self.invincible = False
        self.targetX = spawnX
        self.targetY = spawnY
        self.moving = False
        self.just_finished_move = False
        self.lastCollision: list[arcade.Sprite] = []
        self.debug = PLAYER_DEBUG
        self.debugInterval = PLAYER_DEBUG_INTERVAL
        self.debugTimer = 0
        self.swing_phase = 0.0
        self.flip_timer = 0.0
        self.death_anim_time = 0.0
        self.damage_flash_timer = 0.0

    def updateTexture(self):
        if self.direction == "left":
            if self.dead:
                self.texture = frames[1]
            else:
                self.texture = frames[0]
        else:
            if self.dead:
                self.texture = frames[3]
            else:
                self.texture = frames[2]

    def set_map(self, collisionList, fountainList, spawnX, spawnY):
        self.collisionList = collisionList
        self.fountainList = fountainList
        self.spawnX = spawnX
        self.spawnY = spawnY
        self._placeAt(spawnX, spawnY)

    def reset(self):
        self.hp = self.maxHp
        self.dead = False
        self.facing = "right"
        self.direction = "right"
        self.death_anim_time = 0.0
        self.angle = 0.0
        self.scale_x = 1.0
        self.scale_y = 1.0
        self.alpha = 255
        self.flip_timer = 0.0
        self.damage_flash_timer = 0.0
        self.color = (255, 255, 255)
        self._placeAt(self.spawnX, self.spawnY)

    def set_facing(self, facing: str) -> None:
        if self.dead:
            return
        self.facing = facing
        if facing in ("left", "right"):
            if facing != self.direction:
                self.flip_timer = FLIP_DURATION
                self.swing_phase = 0.0
            self.direction = facing
        self.updateTexture()

    def set_facing_from_key(self, key: int) -> None:
        facing = KEY_TO_FACING.get(key)
        if facing is not None:
            self.set_facing(facing)

    def _update_motion_animation(self, delta_time: float) -> None:
        swing_target = 0.0
        if self.moving and not self.dead:
            swing_sign = -1.0 if self.direction == "left" else 1.0
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
            self.scale_x = abs(math.cos(math.pi * progress))
            if self.flip_timer == 0.0:
                self.scale_x = 1.0

        if self.dead:
            self.color = (255, 255, 255)
            return
        if self.damage_flash_timer > 0.0:
            self.damage_flash_timer = max(
                0.0, self.damage_flash_timer - delta_time
            )
            progress = self.damage_flash_timer / DAMAGE_FLASH_DURATION
            self.color = DAMAGE_FLASH_COLOR
            self.scale_y = 1.0 - 0.12 * progress
        else:
            self.color = (255, 255, 255)
            self.scale_y = 1.0

    def update_death_animation(self, delta_time: float) -> None:
        if not self.dead:
            return
        self.death_anim_time += delta_time
        progress = min(1.0, self.death_anim_time / DEATH_ANIMATION_DURATION)
        self.scale_x = 1.0 + 0.12 * math.sin(math.pi * progress)
        self.scale_y = 1.0 - 0.32 * progress
        self.alpha = int(255 - 90 * progress)
        self.angle = 0.0

    def get_attack_position(self) -> tuple[float, float]:
        offset_x, offset_y = FACING_OFFSETS[self.facing]
        return (
            self.center_x + offset_x * self.gridSize,
            self.center_y + offset_y * self.gridSize,
        )

    def saveState(self) -> dict[str, Any]:
        return {
            "x": self.center_x,
            "y": self.center_y,
            "hp": self.hp,
            "direction": self.direction,
        }

    def loadState(self, state: dict[str, Any]):
        self.hp = max(0, min(int(state["hp"]), self.maxHp))
        self.dead = self.hp == 0
        self.direction = state.get("direction", "right")
        self._placeAt(float(state["x"]), float(state["y"]))

    def _placeAt(self, x, y):
        self.center_x = x
        self.center_y = y
        self.targetX = x
        self.targetY = y
        self.moving = False
        self.just_finished_move = False
        self.updateTexture()

    def takeDamage(self, damage, defense: int = 0) -> int:
        if self.dead or self.invincible:
            return 0

        actual_damage = max(0, damage - defense)
        self.hp -= actual_damage
        if actual_damage > 0:
            self.damage_flash_timer = DAMAGE_FLASH_DURATION

        if self.hp <= 0:
            self.hp = 0
            self.dead = True
            self.moving = False
            self.death_anim_time = 0.0
            self.angle = 0.0
            self.flip_timer = 0.0
            self.scale_x = 1.0
            self.scale_y = 1.0
            self.alpha = 255

        self.updateTexture()
        return actual_damage

    def bumpedFountain(self, collision) -> bool:
        return any(
            sprite in self.fountainList
            for sprite in collision
        )

    def heal(self, amount):
        if self.dead:
            return

        self.hp = min(self.hp + amount, self.maxHp)
        self.updateTexture()

    def collision_with(
        self, sprites: arcade.SpriteList
    ) -> list[arcade.Sprite]:
        old_angle = self.angle
        old_scale_x = self.scale_x
        old_scale_y = self.scale_y
        self.angle = 0.0
        self.scale_x = 1.0
        self.scale_y = 1.0
        collision = arcade.check_for_collision_with_list(self, sprites)
        self.angle = old_angle
        self.scale_x = old_scale_x
        self.scale_y = old_scale_y
        return collision

    def collides_with(self, other: arcade.Sprite) -> bool:
        old_angle = self.angle
        old_scale_x = self.scale_x
        old_scale_y = self.scale_y
        self.angle = 0.0
        self.scale_x = 1.0
        self.scale_y = 1.0
        collision = arcade.check_for_collision(self, other)
        self.angle = old_angle
        self.scale_x = old_scale_x
        self.scale_y = old_scale_y
        return collision

    def checkCollision(self, targetX, targetY):
        oldX = self.center_x
        oldY = self.center_y

        self.center_x = targetX
        self.center_y = targetY
        collision = self.collision_with(self.collisionList)

        self.center_x = oldX
        self.center_y = oldY

        return collision

    def updatePlayer(self, keys, delta_time) -> bool:
        self.just_finished_move = False
        self.lastCollision = []
        self._update_motion_animation(delta_time)

        if self.moving and self.debug:
            self.debugTimer += delta_time

            if self.debugTimer >= self.debugInterval:
                self.debugTimer = 0

                print(
                    f"Player: ({self.center_x:.0f}, {self.center_y:.0f}) | "
                    f"Target: ({self.targetX:.0f}, {self.targetY:.0f}) | "
                    f"Moving: {self.moving} | "
                    f"HP: {self.hp}/{self.maxHp}"
                )

        if self.dead:
            self.updateTexture()
            return False

        if self.moving:
            distanceX = self.targetX - self.center_x
            distanceY = self.targetY - self.center_y

            distance = (distanceX ** 2 + distanceY ** 2) ** 0.5
            moveDistance = self.speed * delta_time

            if distance <= moveDistance:
                self.center_x = self.targetX
                self.center_y = self.targetY
                self.moving = False
                self.just_finished_move = True
            else:
                self.center_x += distanceX / distance * moveDistance
                self.center_y += distanceY / distance * moveDistance

            self.updateTexture()
            return False

        moveX = 0
        moveY = 0

        if arcade.key.W in keys:
            moveY = self.gridSize
            self.set_facing("up")
        elif arcade.key.S in keys:
            moveY = -self.gridSize
            self.set_facing("down")
        elif arcade.key.A in keys:
            moveX = -self.gridSize
            self.set_facing("left")
        elif arcade.key.D in keys:
            moveX = self.gridSize
            self.set_facing("right")

        if moveX == 0 and moveY == 0:
            self.updateTexture()
            return False

        targetX = self.center_x + moveX
        targetY = self.center_y + moveY

        collision = self.checkCollision(targetX, targetY)

        if collision:
            self.lastCollision = collision

            if self.bumpedFountain(collision):
                return True

            if self.debug:
                print(
                    f"COLLISION at "
                    f"({targetX:.0f}, {targetY:.0f}) "
                    f"with {len(collision)} object(s)"
                )

                for index, sprite in enumerate(collision):
                    print(
                        f"  Collision {index}: "
                        f"position=({sprite.center_x:.0f}, "
                        f"{sprite.center_y:.0f}) "
                        f"size=({sprite.width:.0f}x"
                        f"{sprite.height:.0f})"
                    )

            return False

        self.targetX = targetX
        self.targetY = targetY
        self.moving = True

        self.updateTexture()
        return False

    def drawDebug(self):
        self.draw_hit_box()
