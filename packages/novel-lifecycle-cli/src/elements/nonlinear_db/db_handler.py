import argparse
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_subplot = sub.add_parser("add-subplot")
    p_subplot.add_argument("--title", required=True)
    p_subplot.add_argument("--description", default="")

    p_fs = sub.add_parser("add-foreshadowing")
    p_fs.add_argument("--content", required=True)
    p_fs.add_argument("--planted-at", type=int, default=None)

    p_payoff = sub.add_parser("add-payoff")
    p_payoff.add_argument("--foreshadowing-id", type=int, required=True)
    p_payoff.add_argument("--content", required=True)

    p_promise = sub.add_parser("add-promise")
    p_promise.add_argument("--content", required=True)
    p_promise.add_argument("--deadline-chapter", type=int, default=None)

    p_list = sub.add_parser("list")
    p_list.add_argument("--type", required=True, choices=["subplot","foreshadowing","payoff","promise"])

    p_upd = sub.add_parser("update")
    p_upd.add_argument("--type", required=True)
    p_upd.add_argument("--id", type=int, required=True)
    p_upd.add_argument("--status", required=True)

    p_get = sub.add_parser("get")
    p_get.add_argument("--type", required=True)
    p_get.add_argument("--id", type=int, required=True)

    args = parser.parse_args(argv[1:])

    if args.action == "add-subplot":
        rid = db.execute(
            "INSERT INTO subplot (title, description, status) VALUES (?,?,?)",
            (args.title, args.description, "active"),
        )
        db.emit_json(db.query("SELECT * FROM subplot WHERE id=?", (rid,))[0])

    elif args.action == "add-foreshadowing":
        rid = db.execute(
            "INSERT INTO foreshadowing (content, planted_at) VALUES (?,?)",
            (args.content, args.planted_at),
        )
        db.emit_json(db.query("SELECT * FROM foreshadowing WHERE id=?", (rid,))[0])

    elif args.action == "add-payoff":
        rid = db.execute(
            "INSERT INTO payoff (foreshadowing_id, content) VALUES (?,?)",
            (args.foreshadowing_id, args.content),
        )
        db.emit_json(db.query("SELECT * FROM payoff WHERE id=?", (rid,))[0])

    elif args.action == "add-promise":
        rid = db.execute(
            "INSERT INTO promise (content, deadline_chapter) VALUES (?,?)",
            (args.content, args.deadline_chapter),
        )
        db.emit_json(db.query("SELECT * FROM promise WHERE id=?", (rid,))[0])

    elif args.action == "list":
        db.emit_json(db.query(f"SELECT * FROM {args.type}"))

    elif args.action == "update":
        db.execute(f"UPDATE {args.type} SET status=? WHERE id=?", (args.status, args.id))
        db.emit_json({"ok": True, "type": args.type, "id": args.id, "status": args.status})

    elif args.action == "get":
        rows = db.query(f"SELECT * FROM {args.type} WHERE id=?", (args.id,))
        if not rows:
            db.emit_error(f"{args.type} id={args.id} not found")
        db.emit_json(rows[0])

    else:
        parser.print_help()
        sys.exit(1)
