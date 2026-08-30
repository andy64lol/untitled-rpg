from pathlib import Path
import arcade
import sprites


mapPath = Path("src/maps/map1.tmx")

tilemap = arcade.load_tilemap(mapPath)
scene = arcade.Scene.from_tilemap(tilemap)

collisionList = scene["collision"]


class Game(arcade.Window):
    def __init__(self):
        super().__init__(
            800,
            600,
            "Untitled RPG",
        )

        self.camera = arcade.Camera2D()

        self.keys = set()

        self.player = sprites.Player(
            tilemap,
            collisionList,
        )

        self.playerList = arcade.SpriteList()
        self.playerList.append(self.player)

    def on_draw(self):
        self.clear()

        self.camera.use()

        scene.draw()
        self.playerList.draw()

        if self.player.debug:
            self.player.drawDebug()

            for sprite in collisionList:
                sprite.draw_hit_box()

    def on_key_press(self, symbol, modifiers):
        self.keys.add(symbol)

        # Test damage.
        if symbol == arcade.key.H:
            self.player.takeDamage(10)

    def on_key_release(self, symbol, modifiers):
        self.keys.discard(symbol)

    def on_update(self, delta_time):
        self.player.update(
            self.keys,
            delta_time,
        )


game = Game()
arcade.run()
