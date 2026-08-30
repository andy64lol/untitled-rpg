import json
import zlib
from pathlib import Path
from typing import Any

from cryptography.fernet import InvalidToken

from save import cipher


def load_state(path: Path) -> dict[str, Any] | None:
    if not path.exists():
        return None

    try:
        payload = cipher().decrypt(path.read_bytes())
    except InvalidToken:
        return None

    return json.loads(zlib.decompress(payload))
