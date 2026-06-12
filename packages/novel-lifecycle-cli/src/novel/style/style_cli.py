import argparse
import os
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

    p_render = sub.add_parser("render-rule")
    p_render.add_argument("--output", required=True)

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

    elif args.action == "render-rule":
        voices = db.query("SELECT aspect, content FROM style_voice")
        sensitives = db.query("SELECT word, category, severity FROM sensitive_word")
        lines = ["# Style Rules (auto-generated)", ""]
        if voices:
            lines.append("## Voice")
            for v in voices:
                lines.append(f"- **{v['aspect']}**: {v['content']}")
            lines.append("")
        if sensitives:
            lines.append("## Sensitive Words")
            for s in sensitives:
                sev = f" [{s['severity']}]" if s["severity"] else ""
                cat = f" ({s['category']})" if s["category"] else ""
                lines.append(f"- {s['word']}{cat}{sev}")
            lines.append("")
        out_path = args.output
        os.makedirs(os.path.dirname(os.path.abspath(out_path)), exist_ok=True)
        with open(out_path, "w", encoding="utf-8") as f:
            f.write("\n".join(lines))
        db.emit_json({"ok": True, "output": out_path})

    else:
        parser.print_help()
        sys.exit(1)
