from pathlib import Path
import arcade
import sprites


mapPath = Path("src/maps/map1.tmx")

tilemap = arcade.load_tilemap(mapPath)
scene = arcade.Scene.from_tilemap(tilemap)


class Game(arcade.Window):
    def __init__(self):
        super().__init__(800, 600, "Untitled RPG")

        self.camera = arcade.Camera2D()

        self.keys = set()
        self.cameraSpeed = 5

    def on_draw(self):
        self.clear()

        self.camera.use()

        scene.draw()
        sprites.draw()

    def on_key_press(self, symbol, modifiers):
        self.keys.add(symbol)

    def on_key_release(self, symbol, modifiers):
        self.keys.discard(symbol)

    def on_update(self, delta_time):
        x, y = self.camera.position

        if arcade.key.W in self.keys:
            y += self.cameraSpeed
        if arcade.key.S in self.keys:
            y -= self.cameraSpeed
        if arcade.key.A in self.keys:
            x -= self.cameraSpeed
        if arcade.key.D in self.keys:
            x += self.cameraSpeed

        self.camera.position = x, y


game = Game()
arcade.run()
