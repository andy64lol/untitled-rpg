import json
import zlib
from pathlib import Path
from typing import Any

from cryptography.fernet import Fernet

KEY = b"Oys_Zthtdi1x_V49RQCyIOf-Ieu7k8gK-T9YMjdCEmU="


def cipher() -> Fernet:
    return Fernet(KEY)


def save_state(path: Path, state: dict[str, Any]) -> None:
    payload = zlib.compress(json.dumps(state).encode("utf-8"))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(cipher().encrypt(payload))
