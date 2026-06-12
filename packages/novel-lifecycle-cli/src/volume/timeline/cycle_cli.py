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

    p_csig = sub.add_parser("converge-signal")
    p_csig.add_argument("--chapter-id", type=int, required=True)
    p_csig.add_argument("--volume-id", type=int, required=True)
    p_csig.add_argument("--converging", type=int, required=True)
    p_csig.add_argument("--pulled-by-emergent-id", type=int, default=None)
    p_csig.add_argument("--detail", default=None)

    p_cverd = sub.add_parser("convergence-verdict")
    p_cverd.add_argument("--volume-id", type=int, required=True)
    p_cverd.add_argument("--verdict", required=True)
    p_cverd.add_argument("--drift-note", default=None)
    p_cverd.add_argument("--target-achieved", type=int, default=None)

    p_lconv = sub.add_parser("list-convergence")
    p_lconv.add_argument("--volume-id", type=int, required=True)

    p_showv = sub.add_parser("show-volume")
    p_showv.add_argument("volume_id", type=int)

    p_levt = sub.add_parser("list-events")
    p_levt.add_argument("--volume-id", type=int, required=True)
    p_levt.add_argument("--has-node-id", action="store_true")
    p_levt.add_argument("--not-critical", action="store_true")

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

    elif args.action == "converge-signal":
        sid = db.execute(
            "INSERT INTO target_convergence_signal "
            "(chapter_id, volume_id, converging, pulled_by_emergent_id, detail) VALUES (?,?,?,?,?)",
            (args.chapter_id, args.volume_id, args.converging,
             args.pulled_by_emergent_id, args.detail),
        )
        db.emit_json({"ok": True, "id": sid, "chapter_id": args.chapter_id,
                      "converging": args.converging})

    elif args.action == "convergence-verdict":
        vid = args.volume_id
        existing = db.query("SELECT id FROM great_cycles WHERE volume_id=? LIMIT 1", (vid,))
        if existing:
            db.execute(
                "UPDATE great_cycles SET target_convergence_verdict=?, drift_note=? WHERE id=?",
                (args.verdict, args.drift_note, existing[0]["id"]),
            )
            gc_id = existing[0]["id"]
        else:
            gc_id = db.execute(
                "INSERT INTO great_cycles (volume_id, target_convergence_verdict, drift_note) VALUES (?,?,?)",
                (vid, args.verdict, args.drift_note),
            )
        if args.target_achieved is not None:
            db.execute("UPDATE volumes SET target_achieved=? WHERE id=?", (args.target_achieved, vid))
        db.emit_json({"ok": True, "great_cycle_id": gc_id, "volume_id": vid,
                      "verdict": args.verdict})

    elif args.action == "list-convergence":
        rows = db.query(
            "SELECT * FROM target_convergence_signal WHERE volume_id=?",
            (args.volume_id,),
        )
        db.emit_json(rows)

    elif args.action == "show-volume":
        vid = args.volume_id
        vol = db.query("SELECT * FROM volumes WHERE id=?", (vid,))
        if not vol:
            db.emit_error(f"volume {vid} not found")
        events = db.query("SELECT * FROM events WHERE volume_id=?", (vid,))
        gc = db.query("SELECT * FROM great_cycles WHERE volume_id=?", (vid,))
        db.emit_json({"volume": vol[0], "events": events, "great_cycles": gc})

    elif args.action == "list-events":
        conds = ["volume_id=?"]
        params = [args.volume_id]
        if args.has_node_id:
            conds.append("mainplot_node_id IS NOT NULL")
        if args.not_critical:
            conds.append("is_mainplot_critical=0")
        where = " WHERE " + " AND ".join(conds)
        rows = db.query("SELECT * FROM events" + where, tuple(params))
        db.emit_json(rows)

    else:
        parser.print_help()
        sys.exit(1)
