import argparse
import datetime
import json
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_start = sub.add_parser("start")
    p_start.add_argument("chapter_id", type=int)

    p_phase = sub.add_parser("phase")
    p_phase.add_argument("chapter_id", type=int)
    p_phase.add_argument("phase", choices=["birth", "dwell", "decay", "death"])

    p_end = sub.add_parser("end")
    p_end.add_argument("chapter_id", type=int)

    p_res = sub.add_parser("residue")
    p_res.add_argument("residue_action")  # "add"

    args, remaining = parser.parse_known_args(argv[1:])

    now = datetime.datetime.utcnow().isoformat()

    if args.action == "start":
        cid = args.chapter_id
        db.execute(
            "INSERT INTO chapter_writing (chapter_id, started_at) VALUES (?,?) "
            "ON CONFLICT(chapter_id) DO UPDATE SET started_at=excluded.started_at",
            (cid, now),
        )
        db.execute("UPDATE chapters SET status='writing' WHERE id=?", (cid,))
        presummary = db.query("SELECT constraints FROM presummary WHERE chapter_id=?", (cid,))
        constraints = json.loads(presummary[0]["constraints"]) if presummary else []
        db.emit_json({"ok": True, "chapter_id": cid, "constraints": constraints})

    elif args.action == "phase":
        db.execute(
            "INSERT INTO phase_log (chapter_id, phase, logged_at) VALUES (?,?,?)",
            (args.chapter_id, args.phase, now),
        )
        db.emit_json({"ok": True, "chapter_id": args.chapter_id, "phase": args.phase})

    elif args.action == "end":
        cid = args.chapter_id
        presummary = db.query("SELECT constraints FROM presummary WHERE chapter_id=?", (cid,))
        constraints = json.loads(presummary[0]["constraints"]) if presummary else []

        must = [c for c in constraints if c.get("type") == "must" and c.get("mainplot_node_id")]
        if must:
            all_null = all(c.get("satisfied") is None for c in must)
            if all_null:
                db.emit_error("未委派 mainplot-landing-verifier，must 裁决全为 null")
            failed = [c for c in must if c.get("satisfied") == 0]
            if failed:
                db.execute("UPDATE chapters SET status='must_failed' WHERE id=?", (cid,))
                db.emit_error(f"must 约束未落地: {failed}")

        # success
        cw = db.query("SELECT prose_ref FROM chapter_writing WHERE chapter_id=?", (cid,))
        word_count = 0  # telemetry placeholder
        db.execute(
            "UPDATE chapter_writing SET ended_at=?, word_count=? WHERE chapter_id=?",
            (now, word_count, cid),
        )
        db.execute("UPDATE chapters SET status='done' WHERE id=?", (cid,))
        db.emit_json({"ok": True, "chapter_id": cid, "word_count": word_count})

    elif args.action == "residue":
        if args.residue_action == "add":
            p2 = argparse.ArgumentParser()
            p2.add_argument("--chapter-id", type=int, required=True)
            p2.add_argument("--type", required=True)
            p2.add_argument("--content", required=True)
            a2 = p2.parse_args(remaining)
            rid = db.execute(
                "INSERT INTO chapter_residue (chapter_id, residue_type, content) VALUES (?,?,?)",
                (a2.chapter_id, a2.type, a2.content),
            )
            db.emit_json({"ok": True, "id": rid})
        else:
            db.emit_error(f"Unknown residue action: {args.residue_action}")

    else:
        parser.print_help()
        sys.exit(1)
