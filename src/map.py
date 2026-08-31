from pathlib import Path
import arcade


class GameMap:

    def __init__(self, map_path: Path):
        self.path = map_path
        self.tilemap = arcade.load_tilemap(str(map_path))
        self.scene = arcade.Scene.from_tilemap(self.tilemap)
        self.world_width = self.tilemap.width * self.tilemap.tile_width
        self.world_height = self.tilemap.height * self.tilemap.tile_height

        self.spikes = self.scene["spikes"]
        self.fountains = self.scene["fountain"]
        self.collision = self.scene["collision"]

        for fountain in self.fountains:
            self.collision.append(fountain)

    def draw(self, pixelated: bool = False) -> None:
        self.scene.draw(pixelated=pixelated)
