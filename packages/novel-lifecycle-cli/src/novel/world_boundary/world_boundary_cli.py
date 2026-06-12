import argparse
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_add = sub.add_parser("add")
    p_add.add_argument("--type", required=True)
    p_add.add_argument("--content", required=True)
    p_add.add_argument("--fuzzy", type=int, default=0)

    sub.add_parser("list")

    args = parser.parse_args(argv[1:])

    if args.action == "add":
        bid = db.execute(
            "INSERT INTO world_boundaries (boundary_type, content, is_fuzzy) VALUES (?,?,?)",
            (args.type, args.content, args.fuzzy),
        )
        db.emit_json(db.query("SELECT * FROM world_boundaries WHERE id=?", (bid,))[0])

    elif args.action == "list":
        rows = db.query("SELECT * FROM world_boundaries")
        db.emit_json(rows)

    else:
        parser.print_help()
        sys.exit(1)
