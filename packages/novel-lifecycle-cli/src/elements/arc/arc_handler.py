import argparse
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_add = sub.add_parser("add")
    p_add.add_argument("--emergent-id", type=int, required=True)
    p_add.add_argument("--stage", required=True)
    p_add.add_argument("--summary", required=True)

    p_list = sub.add_parser("list")
    p_list.add_argument("--emergent-id", type=int, required=True)

    p_inferred = sub.add_parser("add-inferred")
    p_inferred.add_argument("--emergent-id", type=int, required=True)
    p_inferred.add_argument("--stage", required=True)
    p_inferred.add_argument("--summary", required=True)
    p_inferred.add_argument("--chapter-id", type=int, required=True)

    sub.add_parser("list-dormant-reactivating")

    args = parser.parse_args(argv[1:])

    if args.action == "add":
        rid = db.execute(
            "INSERT INTO arc (emergent_id, stage, summary) VALUES (?,?,?)",
            (args.emergent_id, args.stage, args.summary),
        )
        db.emit_json(db.query("SELECT * FROM arc WHERE id=?", (rid,))[0])

    elif args.action == "list":
        db.emit_json(db.query(
            "SELECT * FROM arc WHERE emergent_id=? ORDER BY id", (args.emergent_id,)
        ))

    elif args.action == "add-inferred":
        rid = db.execute(
            "INSERT INTO arc (emergent_id, stage, summary, is_inferred, inferred_at_chapter_id) VALUES (?,?,?,1,?)",
            (args.emergent_id, args.stage, args.summary, args.chapter_id),
        )
        db.emit_json(db.query("SELECT * FROM arc WHERE id=?", (rid,))[0])

    elif args.action == "list-dormant-reactivating":
        db.emit_json(db.query("SELECT * FROM arc WHERE reactivation_flag=1"))

    else:
        parser.print_help()
        sys.exit(1)
