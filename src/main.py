from pathlib import Path
import arcade
import sprites


mapPath = Path("src/maps/map1.tmx")

tilemap = arcade.load_tilemap(mapPath)
scene = arcade.Scene.from_tilemap(tilemap)

spikesList = scene["spikes"]
collisionList = scene["collision"]

heartTexture = arcade.load_texture("src/assets/UI/heart.png")
brokenHeartTexture = arcade.load_texture("src/assets/UI/broken_heart.png")


class Game(arcade.Window):
    def __init__(self):
        super().__init__(800, 600, "Untitled RPG")

        self.camera = arcade.Camera2D()
        self.guiCamera = arcade.Camera2D()

        self.keys = set()

        self.cameraSpeed = 300

        self.spikeDamageCooldown = 1.0
        self.spikeDamageTimer = 0

        self.player = sprites.Player(tilemap, collisionList)

        self.playerList = arcade.SpriteList()

        self.playerList.append(self.player)

        self.heartList = arcade.SpriteList()

        heartSize = 32
        spacing = 4

        heartCount = self.player.maxHp // 10

        for index in range(heartCount):
            heart = arcade.Sprite(
                heartTexture,
                scale=1,
            )

            heart.center_x = 20 + index * (heartSize + spacing)
            heart.center_y = self.height - 20

            heart.width = heartSize
            heart.height = heartSize

            self.heartList.append(heart)

    def on_draw(self):
        self.clear()

        self.camera.use()

        scene.draw()
        self.playerList.draw()

        self.guiCamera.use()

        self.heartList.draw()

    def updateHealth(self):
        for index, heart in enumerate(self.heartList):
            if self.player.hp > index * 10:
                heart.texture = heartTexture
            else:
                heart.texture = brokenHeartTexture

    def updateHudAlpha(self):
        movingInput = (
            arcade.key.W in self.keys
            or arcade.key.A in self.keys
            or arcade.key.S in self.keys
            or arcade.key.D in self.keys
        )

        if movingInput:
            alpha = 100
        else:
            alpha = 255

        for heart in self.heartList:
            heart.alpha = alpha

    def on_key_press(self, symbol, modifiers):
        self.keys.add(symbol)

        if symbol == arcade.key.H:
            self.player.takeDamage(10)
            self.updateHealth()

    def on_key_release(self, symbol, modifiers):
        self.keys.discard(symbol)

    def on_update(self, delta_time):
        self.player.update(self.keys, delta_time)

        self.spikeDamageTimer -= delta_time

        touchingSpikes = arcade.check_for_collision_with_list(
            self.player,
            spikesList,
        )

        if touchingSpikes and self.spikeDamageTimer <= 0:
            self.player.takeDamage(10)
            self.spikeDamageTimer = self.spikeDamageCooldown
            self.updateHealth()

        if not touchingSpikes:
            self.spikeDamageTimer = 0

        self.updateHudAlpha()

        cameraX = 0
        cameraY = 0

        if arcade.key.LEFT in self.keys:
            cameraX -= self.cameraSpeed * delta_time

        if arcade.key.RIGHT in self.keys:
            cameraX += self.cameraSpeed * delta_time

        if arcade.key.UP in self.keys:
            cameraY += self.cameraSpeed * delta_time

        if arcade.key.DOWN in self.keys:
            cameraY -= self.cameraSpeed * delta_time

        self.camera.position = (
            self.camera.position[0] + cameraX,
            self.camera.position[1] + cameraY,
        )


game = Game()
arcade.run()
