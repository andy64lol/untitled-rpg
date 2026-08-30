from pathlib import Path
import arcade


playerPath = Path("src/assets/sprites/player.png")

spriteSheet = arcade.load_spritesheet(playerPath)

frames = spriteSheet.get_texture_grid(
    size=(32, 32),
    columns=8,
    count=32,
)


class Player(arcade.Sprite):
    def __init__(self, tilemap, collisionList):
        self.gridSize = 32
        self.gridOffset = 16

        spawnX = 400
        spawnY = 300

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
        )

        self.speed = 160

        self.maxHp = 100
        self.hp = self.maxHp

        self.collisionList = collisionList

        self.direction = "right"
        self.dead = False

        self.targetX = spawnX
        self.targetY = spawnY

        self.moving = False

        self.debug = True
        self.debugInterval = 0.25
        self.debugTimer = 0

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

    def takeDamage(self, damage):
        if self.dead:
            return

        self.hp -= damage

        if self.hp <= 0:
            self.hp = 0
            self.dead = True
            self.moving = False

        self.updateTexture()

    def checkCollision(self, targetX, targetY):
        oldX = self.center_x
        oldY = self.center_y

        self.center_x = targetX
        self.center_y = targetY

        collision = arcade.check_for_collision_with_list(
            self,
            self.collisionList,
        )

        self.center_x = oldX
        self.center_y = oldY

        return collision

    def update(self, keys, delta_time):
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
            return

        if self.moving:
            distanceX = self.targetX - self.center_x
            distanceY = self.targetY - self.center_y

            distance = (distanceX ** 2 + distanceY ** 2) ** 0.5
            moveDistance = self.speed * delta_time

            if distance <= moveDistance:
                self.center_x = self.targetX
                self.center_y = self.targetY
                self.moving = False
            else:
                self.center_x += (
                    distanceX / distance * moveDistance
                )
                self.center_y += (
                    distanceY / distance * moveDistance
                )

            self.updateTexture()
            return

        moveX = 0
        moveY = 0

        if arcade.key.W in keys:
            moveY = self.gridSize

        if arcade.key.S in keys:
            moveY = -self.gridSize

        if arcade.key.A in keys:
            moveX = -self.gridSize
            self.direction = "left"

        if arcade.key.D in keys:
            moveX = self.gridSize
            self.direction = "right"

        if moveX == 0 and moveY == 0:
            self.updateTexture()
            return

        targetX = self.center_x + moveX
        targetY = self.center_y + moveY

        collision = self.checkCollision(targetX, targetY)

        if collision:
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

            return

        self.targetX = targetX
        self.targetY = targetY
        self.moving = True

        self.updateTexture()

    def drawDebug(self):
        self.draw_hit_box()
