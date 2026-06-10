import sys
import importlib.util
import os

_DIR = os.path.dirname(os.path.abspath(__file__))

_HANDLERS = {
    "db":       "nonlinear_db/db_handler.py",
    "emergent": "emergent/emergent_handler.py",
    "arc":      "arc/arc_handler.py",
    "bible":    "bible/bible_handler.py",
    "context":  "context/context_handler.py",
}


def _load(rel):
    path = os.path.join(_DIR, rel)
    spec = importlib.util.spec_from_file_location("_handler", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def main(argv):
    if not argv:
        print("element_manager: no subcommand", file=sys.stderr)
        sys.exit(1)
    cmd = argv[0]
    if cmd not in _HANDLERS:
        print(f"element_manager: unknown command '{cmd}'", file=sys.stderr)
        sys.exit(1)
    _load(_HANDLERS[cmd]).main(argv)
