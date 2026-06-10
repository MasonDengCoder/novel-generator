import argparse
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_ce = sub.add_parser("close-event")
    p_ce.add_argument("event_id", type=int)

    p_cv = sub.add_parser("close-volume")
    p_cv.add_argument("volume_id", type=int)

    p_rp = sub.add_parser("repair-plan")
    p_rp.add_argument("level")  # "event"
    p_rp.add_argument("id", type=int)

    args = parser.parse_args(argv[1:])

    if args.action == "close-event":
        eid = args.event_id
        # find chapters under this event that have must constraints and are not done
        blocking = db.query(
            """
            SELECT c.id, c.title, c.status FROM chapters c
            JOIN sections s ON c.section_id = s.id
            WHERE s.event_id = ?
              AND c.status IN ('planned','writing','must_failed')
            """,
            (eid,),
        )
        if blocking:
            db.execute("UPDATE events SET status='repairing' WHERE id=?", (eid,))
            db.emit_error(f"未完成章节阻断封事件: {blocking}", code=1)
        db.execute("UPDATE events SET status='closed' WHERE id=?", (eid,))
        db.emit_json({"ok": True, "event_id": eid})

    elif args.action == "close-volume":
        vid = args.volume_id

        # Gate 1: under-tagged suspects (soft block — list and require confirmation)
        suspects = db.query(
            "SELECT id, title FROM events WHERE volume_id=? AND mainplot_node_id IS NOT NULL AND is_mainplot_critical=0",
            (vid,),
        )
        if suspects:
            db.emit_error(f"欠标嫌疑事件（有 mainplot_node_id 但未标 critical）：{suspects}，请逐条确认", code=1)

        # Gate 2: convergence verdict
        gc = db.query(
            "SELECT target_convergence_verdict FROM great_cycles WHERE volume_id=? LIMIT 1", (vid,)
        )
        if not gc or gc[0]["target_convergence_verdict"] is None:
            db.emit_error("存在性门禁：target_convergence_verdict 缺失，请委派 target-convergence-verifier")

        # Gate 3: reactivation snapshots
        missing_reactivations = db.query(
            """
            SELECT a.id, a.emergent_id FROM arc a
            WHERE a.reactivation_flag=1
              AND NOT EXISTS (
                SELECT 1 FROM arc a2 WHERE a2.emergent_id=a.emergent_id AND a2.is_inferred=1
              )
            """
        )
        if missing_reactivations:
            db.emit_error(f"缺少重激活推测快照，请先委派 reactivation-inferer: {missing_reactivations}")

        # Gate 4: all critical events closed
        unclosed = db.query(
            "SELECT id, title, status FROM events WHERE volume_id=? AND is_mainplot_critical=1 AND status!='closed'",
            (vid,),
        )
        if unclosed:
            db.emit_error(f"存在未封闭的必达事件: {unclosed}")

        # Gate 5: decomposition audit summary (informational output, requires ACK outside CLI)
        audit = db.query(
            "SELECT * FROM mainplot_decomposition_audit WHERE volume_id=?", (vid,)
        )
        db.execute(
            "UPDATE volumes SET status='sealed', target_achieved=1 WHERE id=?", (vid,)
        )
        db.emit_json({"ok": True, "volume_id": vid, "decomposition_audit": audit})

    elif args.action == "repair-plan":
        if args.level != "event":
            db.emit_error(f"不支持的层级: {args.level}")
        eid = args.id
        chapters = db.query(
            """
            SELECT c.id, c.title, c.status, p.constraints
            FROM chapters c
            JOIN sections s ON c.section_id=s.id
            LEFT JOIN presummary p ON p.chapter_id=c.id
            WHERE s.event_id=? AND c.status='must_failed'
            """,
            (eid,),
        )
        db.emit_json(chapters)

    else:
        parser.print_help()
        sys.exit(1)
