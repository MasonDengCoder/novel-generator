import argparse
import sys
from core import db


_STAGES = ["inspiration", "world-boundary", "main-plot", "destiny"]


def _ensure_stages():
    for s in _STAGES:
        existing = db.query("SELECT 1 FROM stage_progress WHERE stage_name=?", (s,))
        if not existing:
            db.execute("INSERT INTO stage_progress (stage_name, status) VALUES (?,?)", (s, "pending"))


def _ensure_stage_data_table():
    """Create stage_data table if not exists (stores key-value pairs for stage artifacts)."""
    db.execute(
        "CREATE TABLE IF NOT EXISTS stage_data "
        "(key TEXT PRIMARY KEY, content TEXT NOT NULL)"
    )


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_complete = sub.add_parser("complete")
    p_complete.add_argument("stage")

    sub.add_parser("list")

    p_set_insp = sub.add_parser("set-inspiration")
    p_set_insp.add_argument("--content", required=True)

    sub.add_parser("get-inspiration")

    p_set_prem = sub.add_parser("set-premise")
    p_set_prem.add_argument("--content", required=True)

    sub.add_parser("get-premise")

    args = parser.parse_args(argv[1:])

    _ensure_stages()
    _ensure_stage_data_table()

    if args.action == "list":
        rows = db.query("SELECT stage_name, status, completed_at FROM stage_progress")
        db.emit_json(rows)

    elif args.action == "complete":
        stage = args.stage
        if stage not in _STAGES:
            db.emit_error(f"未知阶段: {stage}")

        if stage == "world-boundary":
            sig = db.query(
                "SELECT 1 FROM finalcheck_signal WHERE check_type='boundary_validated' AND result='pass' LIMIT 1"
            )
            if not sig:
                db.emit_error("存在性门禁：boundary_validated 缺失，请先委派 world-boundary-validator")

        existing = db.query("SELECT status FROM stage_progress WHERE stage_name=?", (stage,))
        if not existing:
            db.emit_error(f"阶段不存在: {stage}")
        if existing[0]["status"] == "completed":
            db.emit_json({"ok": True, "stage": stage, "status": "already_completed"})
            return

        import datetime
        now = datetime.datetime.utcnow().isoformat()
        db.execute(
            "UPDATE stage_progress SET status='completed', completed_at=? WHERE stage_name=?",
            (now, stage),
        )
        db.emit_json({"ok": True, "stage": stage, "completed_at": now})
    elif args.action == "set-inspiration":
        db.execute(
            "INSERT INTO stage_data (key, content) VALUES ('inspiration', ?) "
            "ON CONFLICT(key) DO UPDATE SET content=excluded.content",
            (args.content,),
        )
        db.emit_json({"ok": True, "key": "inspiration"})

    elif args.action == "get-inspiration":
        rows = db.query("SELECT content FROM stage_data WHERE key='inspiration'")
        if not rows:
            db.emit_error("inspiration not set")
        db.emit_json({"key": "inspiration", "content": rows[0]["content"]})

    elif args.action == "set-premise":
        db.execute(
            "INSERT INTO stage_data (key, content) VALUES ('premise', ?) "
            "ON CONFLICT(key) DO UPDATE SET content=excluded.content",
            (args.content,),
        )
        db.emit_json({"ok": True, "key": "premise"})

    elif args.action == "get-premise":
        rows = db.query("SELECT content FROM stage_data WHERE key='premise'")
        if not rows:
            db.emit_error("premise not set")
        db.emit_json({"key": "premise", "content": rows[0]["content"]})

    else:
        parser.print_help()
        sys.exit(1)
