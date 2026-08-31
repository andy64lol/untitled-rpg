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
        self.chests = self.scene["chest"]
        self.door = self.scene["door"]
        self.door.visible = True

        for fountain in self.fountains:
            self.collision.append(fountain)
        for chest in self.chests:
            self.collision.append(chest)
        for door in self.door:
            self.collision.append(door)

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
                (door.center_x - target_x) ** 2
                + (door.center_y - target_y) ** 2
            )
            ** 0.5
            <= 40
            for door in self.door
        )

    def unlock_door(self) -> None:
        self.door.visible = False
        for door in list(self.door):
            if door in self.collision:
                self.collision.remove(door)

    def lock_door(self) -> None:
        self.door.visible = True
        for door in self.door:
            if door not in self.collision:
                self.collision.append(door)
