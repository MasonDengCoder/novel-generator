import argparse
import datetime
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    # weave volume
    p_vol = sub.add_parser("volume")
    p_vol.add_argument("--title", required=True)
    p_vol.add_argument("--mainplot-target-node-id", type=int, default=None)

    # weave event
    p_evt = sub.add_parser("event")
    p_evt.add_argument("--volume-id", type=int, required=True)
    p_evt.add_argument("--title", required=True)
    p_evt.add_argument("--mainplot-node-id", type=int, default=None)
    p_evt.add_argument("--is-mainplot-critical", type=int, default=0)

    # weave section
    p_sec = sub.add_parser("section")
    p_sec.add_argument("--event-id", type=int, required=True)
    p_sec.add_argument("--title", required=True)
    p_sec.add_argument("--mainplot-must-beat", default=None)

    # weave chapter
    p_ch = sub.add_parser("chapter")
    p_ch.add_argument("--section-id", type=int, required=True)
    p_ch.add_argument("--seq", type=int, required=True)
    p_ch.add_argument("--title", required=True)

    args = parser.parse_args(argv[1:])
    now = datetime.datetime.utcnow().isoformat()

    if args.action == "volume":
        order = (db.query("SELECT COUNT(*) as c FROM volumes")[0]["c"] or 0) + 1
        vid = db.execute(
            "INSERT INTO volumes (title, status, \"order\", mainplot_target_node_id) VALUES (?,?,?,?)",
            (args.title, "open", order, args.mainplot_target_node_id),
        )
        if args.mainplot_target_node_id:
            db.execute(
                "INSERT INTO mainplot_decomposition_audit (mainplot_node_id, volume_id, created_at) VALUES (?,?,?)",
                (args.mainplot_target_node_id, vid, now),
            )
        rows = db.query("SELECT * FROM volumes WHERE id=?", (vid,))
        db.emit_json(rows[0])

    elif args.action == "event":
        eid = db.execute(
            "INSERT INTO events (volume_id, title, status, mainplot_node_id, is_mainplot_critical) VALUES (?,?,?,?,?)",
            (args.volume_id, args.title, "open", args.mainplot_node_id, args.is_mainplot_critical),
        )
        if args.mainplot_node_id and args.is_mainplot_critical:
            db.execute(
                "INSERT INTO mainplot_decomposition_audit (mainplot_node_id, event_id, volume_id, created_at) VALUES (?,?,?,?)",
                (args.mainplot_node_id, eid, args.volume_id, now),
            )
        rows = db.query("SELECT * FROM events WHERE id=?", (eid,))
        db.emit_json(rows[0])

    elif args.action == "section":
        sid = db.execute(
            "INSERT INTO sections (event_id, title, mainplot_must_beat) VALUES (?,?,?)",
            (args.event_id, args.title, args.mainplot_must_beat),
        )
        rows = db.query("SELECT * FROM sections WHERE id=?", (sid,))
        db.emit_json(rows[0])

    elif args.action == "chapter":
        # rolling window: find which volume this section belongs to
        vol_info = db.query(
            """
            SELECT v.id, v."order" FROM volumes v
            JOIN events e ON e.volume_id=v.id
            JOIN sections s ON s.event_id=e.id
            WHERE s.id=?
            """,
            (args.section_id,),
        )
        if vol_info:
            vol_order = vol_info[0]["order"]
            # volumes beyond position 2 (index 0-based) = 3rd+ → event-level only
            current_vol_count = db.query("SELECT COUNT(*) as c FROM volumes")[0]["c"]
            # find min order of "current" (first open) volume
            open_vols = db.query(
                "SELECT \"order\" FROM volumes WHERE status='open' ORDER BY \"order\" ASC LIMIT 1"
            )
            if open_vols:
                current_order = open_vols[0]["order"]
                if vol_order > current_order + 1:
                    db.emit_error(f"第三卷及以后只允许事件级编织，不可添加章节到该卷")

        cid = db.execute(
            "INSERT INTO chapters (section_id, seq, title, status) VALUES (?,?,?,?)",
            (args.section_id, args.seq, args.title, "planned"),
        )
        rows = db.query("SELECT * FROM chapters WHERE id=?", (cid,))
        db.emit_json(rows[0])

    else:
        parser.print_help()
        sys.exit(1)
