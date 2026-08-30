from pathlib import Path
import arcade


class GameMap:

    def __init__(self, map_path: Path):
        self.path = map_path
        self.tilemap = arcade.load_tilemap(str(map_path))
        self.scene = arcade.Scene.from_tilemap(self.tilemap)

        self.spikes = self.scene["spikes"]
        self.fountains = self.scene["fountain"]
        self.collision = self.scene["collision"]

        for fountain in self.fountains:
            self.collision.append(fountain)

    def draw(self) -> None:
        self.scene.draw()
