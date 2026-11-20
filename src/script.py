from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from config import *
from items import ITEMS

_SPECIAL = set("{}();<>!")
_OPERATORS = (">=", "<=", "==", "!=", ">", "<")


def _compare(value: Any, operator: str, raw: str) -> bool:
    if isinstance(value, str):
        right: Any = raw
    else:
        try:
            right = float(raw)
        except ValueError:
            right = raw
    if operator == ">":
        return value > right
    if operator == "<":
        return value < right
    if operator == ">=":
        return value >= right
    if operator == "<=":
        return value <= right
    if operator == "!=":
        return value != right
    return value == right


@dataclass
class Node:
    command: str
    args: list[str]
    children: list["Node"] | None = None
    else_children: list["Node"] | None = None


@dataclass
class ScriptRunner:
    name: str
    nodes: list[Node]
    frames: list[tuple[list[Node], int]] = field(default_factory=list)
    nested: list["ScriptRunner"] = field(default_factory=list)
    wait: float = 0.0
    waiting_dialogue: bool = False

    @classmethod
    def load(cls, name_or_path: str) -> "ScriptRunner":
        path = Path(name_or_path)
        if not path.is_absolute() and not path.exists():
            path = SCRIPTS_DIR / f"{name_or_path}{SCRIPT_EXTENSION}"
        text = path.read_text(encoding="utf-8")
        nodes = parse_script(text)
        runner = cls(name=path.stem, nodes=nodes)
        runner.frames = [(nodes, 0)]
        return runner

    @staticmethod
    def _item_count(game: Any, item_id: str) -> int:
        return sum(
            stack.count
            for stack in game.inventory.slots
            if stack is not None and stack.item_id == item_id
        )

    def _condition(self, args: list[str], game: Any) -> bool:
        if not args:
            return False
        kind = args[0].lower()
        value: Any = False
        offset = 1

        if kind.startswith("player."):
            member = kind.split(".", 1)[1]
            if member in ("health", "hp"):
                value = game.player.hp
            elif member in ("maxhealth", "maxhp"):
                value = game.player.maxHp
            elif member == "dead":
                value = game.player.dead
            elif member == "moving":
                value = game.player.moving
            elif member == "facing":
                value = game.player.facing
            elif member == "gold":
                value = game.gold
            elif member == "map":
                value = Path(game.map_name).stem
            elif member == "item" and len(args) >= 2:
                value = self._item_count(game, args[1])
                offset = 2
            else:
                return False
        elif kind == "item" and len(args) >= 2:
            value = self._item_count(game, args[1])
            offset = 2
        elif kind == "gold":
            value = game.gold
        elif kind == "flag" and len(args) >= 2:
            value = args[1] in game.script_flags
            offset = 2
        elif kind == "map":
            value = Path(game.map_name).stem
        else:
            return False

        if len(args) <= offset:
            return bool(value)
        operator = args[offset]
        if operator in _OPERATORS:
            if len(args) <= offset + 1:
                return bool(value)
            return _compare(value, operator, args[offset + 1])
        return _compare(value, "==" if isinstance(value, str) else ">=", operator)

    def _execute(self, node: Node, game: Any) -> None:
        command = node.command
        args = node.args
        if command == "give" and len(args) >= 1:
            count = int(args[1]) if len(args) >= 2 else 1
            if args[0] in ITEMS:
                game.inventory.add(args[0], count)
                game.inventory_screen.refresh()
                game.unsaved = True
        elif command == "take" and len(args) >= 1:
            count = int(args[1]) if len(args) >= 2 else 1
            if game.inventory.remove_item(args[0], count):
                game.inventory_screen.refresh()
                game.unsaved = True
        elif command == "gold" and len(args) >= 1:
            game.gold = max(0, game.gold + int(args[0]))
            game.inventory_screen.set_gold(game.gold)
            game.unsaved = True
        elif command == "setgold" and len(args) >= 1:
            game.gold = max(0, int(args[0]))
            game.inventory_screen.set_gold(game.gold)
            game.unsaved = True
        elif command == "heal" and len(args) >= 1:
            game.player.heal(int(args[0]))
            game.update_health()
            game.unsaved = True
        elif command == "damage" and len(args) >= 1:
            dealt = game.player.takeDamage(int(args[0]), 0)
            game.spawn_damage_number(dealt)
            game.update_health()
            game.unsaved = True
        elif command == "sethealth" and len(args) >= 1:
            game.player.hp = max(0, min(int(args[0]), game.player.maxHp))
            game.update_health()
            game.unsaved = True
        elif command == "spawn" and len(args) >= 1:
            tile_width = game.game_map.tilemap.tile_width
            tile_height = game.game_map.tilemap.tile_height
            if len(args) >= 3:
                x = int(args[1]) * tile_width + tile_width / 2
                y = int(args[2]) * tile_height + tile_height / 2
            else:
                x = game.player.center_x + tile_width
                y = game.player.center_y
            game.enemies.spawn(args[0], x, y)
        elif command == "flag" and len(args) >= 1:
            game.script_flags.add(args[0])
            game.unsaved = True
        elif command == "unflag" and len(args) >= 1:
            game.script_flags.discard(args[0])
            game.unsaved = True

    def update(self, delta_time: float, game: Any) -> bool:
        if self.nested:
            if self.nested[-1].update(delta_time, game):
                self.nested.pop()
            return False

        if self.wait > 0.0:
            self.wait = max(0.0, self.wait - delta_time)
            return False

        if self.waiting_dialogue:
            if game.dialogue.is_open:
                return False
            self.waiting_dialogue = False

        while self.frames:
            nodes, index = self.frames[-1]
            if index >= len(nodes):
                self.frames.pop()
                continue
            node = nodes[index]
            self.frames[-1] = (nodes, index + 1)

            if node.command == "if":
                branch = (
                    node.children
                    if self._condition(node.args, game)
                    else node.else_children
                )
                if branch:
                    self.frames.append((branch, 0))
                continue

            if node.command == "wait" and node.args:
                self.wait = max(0.0, float(node.args[0]))
                return False
            if node.command in ("dialogue", "say"):
                game.dialogue.show(" ".join(node.args))
                self.waiting_dialogue = True
                return False
            if node.command == "run" and node.args:
                self.nested.append(ScriptRunner.load(node.args[0]))
                return False
            if node.command == "end":
                self.frames.clear()
                return True
            self._execute(node, game)

        return True


def tokenize(text: str) -> list[str]:
    tokens: list[str] = []
    index = 0
    while index < len(text):
        char = text[index]
        if char == "\n":
            tokens.append("NL")
            index += 1
        elif char.isspace() or char == ",":
            index += 1
        elif char == "#" or (
            char == "/" and index + 1 < len(text) and text[index + 1] == "/"
        ):
            while index < len(text) and text[index] != "\n":
                index += 1
        elif char in ('"', "'"):
            quote = char
            index += 1
            value = ""
            while index < len(text) and text[index] != quote:
                if text[index] == "\\" and index + 1 < len(text):
                    value += text[index + 1]
                    index += 2
                else:
                    value += text[index]
                    index += 1
            if index < len(text):
                index += 1
            tokens.append(value)
        elif char in _SPECIAL:
            pair = text[index : index + 2]
            if pair in _OPERATORS:
                tokens.append(pair)
                index += 2
            else:
                tokens.append(char)
                index += 1
        else:
            start = index
            while (
                index < len(text)
                and not text[index].isspace()
                and text[index] not in _SPECIAL
                and text[index] != "#"
                and text[index] != ","
            ):
                index += 1
            tokens.append(text[start:index])
    return tokens


def parse_script(text: str) -> list[Node]:
    tokens = tokenize(text)
    position = 0

    def peek() -> str | None:
        return tokens[position] if position < len(tokens) else None

    def take() -> str:
        nonlocal position
        token = tokens[position]
        position += 1
        return token

    def skip_separators() -> None:
        while peek() in ("NL", ";"):
            take()

    def parse_if() -> Node:
        if peek() == "(":
            take()
        condition: list[str] = []
        while peek() is not None and peek() != ")":
            condition.append(take())
        if peek() == ")":
            take()
        if peek() == "{":
            take()
        children = parse_block()
        if peek() == "}":
            take()
        else_children: list[Node] | None = None
        next_token = peek()
        if next_token is not None and next_token.lower() == "else":
            take()
            after_else = peek()
            if after_else is not None and after_else.lower() == "if":
                else_children = [parse_if()]
            else:
                if peek() == "{":
                    take()
                else_children = parse_block()
                if peek() == "}":
                    take()
        return Node("if", condition, children, else_children)

    def parse_block() -> list[Node]:
        nodes: list[Node] = []
        skip_separators()
        while peek() is not None and peek() != "}":
            first = take()
            if first.lower() == "if":
                nodes.append(parse_if())
            else:
                args: list[str] = []
                if peek() == "(":
                    take()
                    while peek() is not None and peek() != ")":
                        args.append(take())
                    if peek() == ")":
                        take()
                else:
                    while (
                        peek() is not None
                        and peek() not in ("NL", ";", "{", "}")
                    ):
                        args.append(take())
                if peek() in ("NL", ";"):
                    take()
                nodes.append(Node(first.lower(), args))
            skip_separators()
        return nodes

    return parse_block()
