import argparse

import arcade

from config import MAP_FILES
from game import GameWindow


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Untitled RPG")
    level_group = parser.add_mutually_exclusive_group()
    for index in range(len(MAP_FILES)):
        level_group.add_argument(
            f"--lv{index + 1}",
            dest="level",
            action="store_const",
            const=index,
            help=f"Start directly on map {index + 1}",
        )

    parser.add_argument(
        "--health-infinite",
        dest="health_infinite",
        action="store_true",
        help="Make the player immortal (ignore all damage)",
    )

    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> None:
    args = parse_args(argv)
    GameWindow(start_level=args.level, health_infinite=args.health_infinite)
    arcade.run()


if __name__ == "__main__":
    main()
