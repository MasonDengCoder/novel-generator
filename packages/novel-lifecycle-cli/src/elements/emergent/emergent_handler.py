import argparse
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_add = sub.add_parser("add")
    p_add.add_argument("--type", required=True)
    p_add.add_argument("--name", required=True)
    p_add.add_argument("--created-chapter-id", type=int, required=True)

    p_list = sub.add_parser("list")
    p_list.add_argument("--status", default=None)

    p_score = sub.add_parser("score")
    p_score.add_argument("--id", type=int, required=True)
    p_score.add_argument("--causal", type=float, required=True)
    p_score.add_argument("--character", type=float, required=True)
    p_score.add_argument("--theme", type=float, required=True)
    p_score.add_argument("--surprise", type=float, required=True)
    p_score.add_argument("--chapter-id", type=int, required=True)

    p_disp = sub.add_parser("update-disposition")
    p_disp.add_argument("--id", type=int, required=True)
    p_disp.add_argument("--disposition", required=True)
    p_disp.add_argument("--level", required=True)

    args = parser.parse_args(argv[1:])

    if args.action == "add":
        rid = db.execute(
            "INSERT INTO emergent (type, name, status, created_chapter_id) VALUES (?,?,?,?)",
            (args.type, args.name, "pending", args.created_chapter_id),
        )
        db.emit_json(db.query("SELECT * FROM emergent WHERE id=?", (rid,))[0])

    elif args.action == "list":
        if args.status:
            rows = db.query("SELECT * FROM emergent WHERE status=?", (args.status,))
        else:
            rows = db.query("SELECT * FROM emergent")
        db.emit_json(rows)

    elif args.action == "score":
        composite = (args.causal * 0.4 + args.character * 0.3 +
                     args.theme * 0.2 + args.surprise * 0.1)
        db.execute(
            """UPDATE emergent SET score_causal=?, score_character=?, score_theme=?,
               score_surprise=?, score=?, scored_at_chapter_id=? WHERE id=?""",
            (args.causal, args.character, args.theme, args.surprise,
             composite, args.chapter_id, args.id),
        )
        db.emit_json({"ok": True, "id": args.id, "score": composite})

    elif args.action == "update-disposition":
        db.execute(
            "UPDATE emergent SET disposition=?, disposition_level=? WHERE id=?",
            (args.disposition, args.level, args.id),
        )
        db.emit_json({"ok": True, "id": args.id})

    else:
        parser.print_help()
        sys.exit(1)
