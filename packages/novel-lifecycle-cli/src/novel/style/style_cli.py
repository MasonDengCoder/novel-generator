import argparse
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_voice = sub.add_parser("set-voice")
    p_voice.add_argument("--aspect", required=True)
    p_voice.add_argument("--content", required=True)

    p_add = sub.add_parser("add-sensitive")
    p_add.add_argument("--word", required=True)
    p_add.add_argument("--category", default="")
    p_add.add_argument("--severity", default="")

    sub.add_parser("list-sensitive")

    args = parser.parse_args(argv[1:])

    if args.action == "set-voice":
        db.execute(
            "INSERT INTO style_voice (aspect, content) VALUES (?,?) "
            "ON CONFLICT(aspect) DO UPDATE SET content=excluded.content",
            (args.aspect, args.content),
        )
        db.emit_json({"ok": True, "aspect": args.aspect})

    elif args.action == "add-sensitive":
        row_id = db.execute(
            "INSERT INTO sensitive_word (word, category, severity) VALUES (?,?,?)",
            (args.word, args.category, args.severity),
        )
        db.emit_json({"ok": True, "id": row_id})

    elif args.action == "list-sensitive":
        rows = db.query("SELECT id, word, category, severity FROM sensitive_word")
        db.emit_json(rows)

    else:
        parser.print_help()
        sys.exit(1)
