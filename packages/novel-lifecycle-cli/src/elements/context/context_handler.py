import argparse
import sys
from core import db

_ACTION_PRIORITY = {"payoff": 1, "advance_milestone": 2, "reactivate": 3, "advance": 4, "plant": 5}


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_res = sub.add_parser("_resolve-chapter")
    p_res.add_argument("seq", type=int)

    p_ch = sub.add_parser("chapter")
    p_ch.add_argument("chapter_id", type=int)
    p_ch.add_argument("--limit", type=int, default=15)

    p_up = sub.add_parser("upcoming")
    p_up.add_argument("chapter_id", type=int)
    p_up.add_argument("--limit", type=int, default=10)

    p_react = sub.add_parser("reactivations")
    p_react.add_argument("chapter_id", type=int)

    p_bind = sub.add_parser("bind")
    p_bind.add_argument("--chapter-id", type=int, required=True)
    p_bind.add_argument("--element-type", required=True)
    p_bind.add_argument("--element-id", type=int, required=True)
    p_bind.add_argument("--relevance", type=float, required=True)
    p_bind.add_argument("--action", required=True)

    args = parser.parse_args(argv[1:])

    if args.action == "_resolve-chapter":
        rows = db.query("SELECT id FROM chapters WHERE seq=? LIMIT 1", (args.seq,))
        if not rows:
            db.emit_error(f"chapter seq={args.seq} not found")
        db.emit_json({"chapter_id": rows[0]["id"]})

    elif args.action == "chapter":
        rows = db.query(
            """SELECT * FROM chapter_threads WHERE chapter_id=?
               ORDER BY action, relevance DESC LIMIT ?""",
            (args.chapter_id, args.limit),
        )
        # sort by action priority
        rows.sort(key=lambda r: (_ACTION_PRIORITY.get(r["action"], 99), -r["relevance"]))
        db.emit_json(rows[:args.limit])

    elif args.action == "upcoming":
        rows = db.query(
            """SELECT ct.*, p.deadline_chapter FROM chapter_threads ct
               JOIN promise p ON ct.element_type='promise' AND ct.element_id=p.id
               WHERE ct.chapter_id=? AND ct.action='payoff'
                 AND p.deadline_chapter <= ? + 5
               ORDER BY p.deadline_chapter ASC LIMIT ?""",
            (args.chapter_id, args.chapter_id, args.limit),
        )
        db.emit_json(rows)

    elif args.action == "reactivations":
        rows = db.query(
            """SELECT ct.*, a.summary, a.inferred_at_chapter_id FROM chapter_threads ct
               JOIN arc a ON ct.element_type='arc' AND ct.element_id=a.id
               WHERE ct.chapter_id=? AND ct.action='reactivate' AND a.is_inferred=1""",
            (args.chapter_id,),
        )
        db.emit_json(rows)

    elif args.action == "bind":
        rid = db.execute(
            "INSERT INTO chapter_threads (chapter_id, element_type, element_id, relevance, action) VALUES (?,?,?,?,?)",
            (args.chapter_id, args.element_type, args.element_id, args.relevance, args.action),
        )
        db.emit_json({"ok": True, "id": rid})

    else:
        parser.print_help()
        sys.exit(1)
