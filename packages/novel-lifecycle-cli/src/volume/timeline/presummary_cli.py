import argparse
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_set = sub.add_parser("set")
    p_set.add_argument("--chapter-id", type=int, required=True)
    p_set.add_argument("--constraints", required=True)

    p_get = sub.add_parser("get")
    p_get.add_argument("--chapter-id", type=int, required=True)

    p_upd = sub.add_parser("update-verdict")
    p_upd.add_argument("--chapter-id", type=int, required=True)
    p_upd.add_argument("--constraint-idx", type=int, required=True)
    p_upd.add_argument("--satisfied", type=int, required=True)
    p_upd.add_argument("--reason", required=True)
    p_upd.add_argument("--verified-by", required=True)

    p_uc = sub.add_parser("update-constraint")
    p_uc.add_argument("--chapter-id", type=int, required=True)
    p_uc.add_argument("--constraint-idx", type=int, required=True)
    p_uc.add_argument("--content", required=True)

    args = parser.parse_args(argv[1:])

    if args.action == "set":
        db.execute(
            "INSERT INTO presummary (chapter_id, constraints) VALUES (?,?) "
            "ON CONFLICT(chapter_id) DO UPDATE SET constraints=excluded.constraints",
            (args.chapter_id, args.constraints),
        )
        db.emit_json({"ok": True, "chapter_id": args.chapter_id})

    elif args.action == "get":
        rows = db.query("SELECT * FROM presummary WHERE chapter_id=?", (args.chapter_id,))
        if not rows:
            db.emit_error(f"presummary not found for chapter {args.chapter_id}")
        db.emit_json(rows[0])

    elif args.action == "update-verdict":
        if len(args.reason) < 10:
            db.emit_error("verdict_reason 不得少于10字符")
        if not args.verified_by:
            db.emit_error("verified_by 不能为空")

        rows = db.query("SELECT constraints FROM presummary WHERE chapter_id=?", (args.chapter_id,))
        if not rows:
            db.emit_error(f"presummary not found for chapter {args.chapter_id}")

        import json
        constraints = json.loads(rows[0]["constraints"])
        idx = args.constraint_idx
        if idx < 0 or idx >= len(constraints):
            db.emit_error(f"constraint index {idx} out of range")

        constraints[idx]["satisfied"] = args.satisfied
        constraints[idx]["verdict_reason"] = args.reason
        constraints[idx]["verified_by"] = args.verified_by

        db.execute(
            "UPDATE presummary SET constraints=? WHERE chapter_id=?",
            (json.dumps(constraints, ensure_ascii=False), args.chapter_id),
        )
        db.emit_json({"ok": True, "chapter_id": args.chapter_id, "constraint_idx": idx})

    elif args.action == "update-constraint":
        rows = db.query("SELECT constraints FROM presummary WHERE chapter_id=?", (args.chapter_id,))
        if not rows:
            db.emit_error(f"presummary not found for chapter {args.chapter_id}")

        import json
        constraints = json.loads(rows[0]["constraints"])
        idx = args.constraint_idx
        if idx < 0 or idx >= len(constraints):
            db.emit_error(f"constraint index {idx} out of range")

        constraints[idx]["content"] = args.content

        db.execute(
            "UPDATE presummary SET constraints=? WHERE chapter_id=?",
            (json.dumps(constraints, ensure_ascii=False), args.chapter_id),
        )
        db.emit_json({"ok": True, "chapter_id": args.chapter_id, "constraint_idx": idx})

    else:
        parser.print_help()
        sys.exit(1)
