import argparse
import sys
from datetime import datetime
from typing import TextIO

import arcade

from config import LOGS_DIR, MAP_FILES
from game import GameWindow


class _Tee:
    def __init__(self, console: TextIO, log_file: TextIO) -> None:
        self.console = console
        self.log_file = log_file

    def write(self, text: str) -> int:
        self.console.write(text)
        self.log_file.write(text)
        self.console.flush()
        self.log_file.flush()
        return len(text)

    def flush(self) -> None:
        self.console.flush()
        self.log_file.flush()


def open_log_file() -> TextIO:
    LOGS_DIR.mkdir(parents=True, exist_ok=True)
    number = 1
    while (LOGS_DIR / f"run{number}.txt").exists():
        number += 1
    return (LOGS_DIR / f"run{number}.txt").open("w", encoding="utf-8")


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

    parser.add_argument(
        "--disable-player-light",
        dest="disable_player_light",
        action="store_true",
        help="Disable the player's own vision light",
    )

    parser.add_argument(
        "--freecam",
        dest="freecam",
        action="store_true",
        help="Detach the camera from the player (pan with arrow keys)",
    )

    parser.add_argument(
        "--disable-map-light",
        dest="disable_map_light",
        action="store_true",
        help="Disable light sources placed in the map",
    )

    parser.add_argument(
        "--output-file",
        dest="output_file",
        action="store_true",
        help="Write all logs to logs/runX.txt in addition to the terminal",
    )

    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> None:
    args = parse_args(argv)
    if args.output_file:
        log_file = open_log_file()
        raw_args = sys.argv[1:] if argv is None else argv
        flags = " ".join(raw_args) or "(none)"
        log_file.write("=" * 60 + "\n")
        log_file.write(f"log:   {log_file.name}\n")
        log_file.write(f"date:  {datetime.now():%Y-%m-%d %H:%M:%S}\n")
        log_file.write(f"flags: {flags}\n")
        log_file.write("=" * 60 + "\n")
        log_file.flush()
        sys.stdout = _Tee(sys.stdout, log_file)
    GameWindow(
        start_level=args.level,
        health_infinite=args.health_infinite,
        disable_player_light=args.disable_player_light,
        freecam=args.freecam,
        disable_map_light=args.disable_map_light,
    )
    arcade.run()


if __name__ == "__main__":
    main()
