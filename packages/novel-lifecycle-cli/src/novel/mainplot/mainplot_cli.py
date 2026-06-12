import argparse
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_add = sub.add_parser("add-node")
    p_add.add_argument("--content", required=True)
    p_add.add_argument("--immutable", type=int, default=0)
    p_add.add_argument("--order", type=int, default=None)

    p_list = sub.add_parser("list")
    p_list.add_argument("--immutable-only", action="store_true")

    args = parser.parse_args(argv[1:])

    if args.action == "add-node":
        nid = db.execute(
            "INSERT INTO mainplot_nodes (content, is_immutable, \"order\") VALUES (?,?,?)",
            (args.content, args.immutable, args.order),
        )
        db.emit_json(db.query("SELECT * FROM mainplot_nodes WHERE node_id=?", (nid,))[0])

    elif args.action == "list":
        if args.immutable_only:
            rows = db.query("SELECT * FROM mainplot_nodes WHERE is_immutable=1 ORDER BY \"order\"")
        else:
            rows = db.query("SELECT * FROM mainplot_nodes ORDER BY \"order\"")
        db.emit_json(rows)

    else:
        parser.print_help()
        sys.exit(1)
