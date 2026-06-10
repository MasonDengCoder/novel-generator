import argparse
import datetime
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    sub.add_parser("check")
    sub.add_parser("seal")

    args = parser.parse_args(argv[1:])

    if args.action == "check":
        orphans = db.query(
            """
            SELECT node_id, content FROM mainplot_nodes
            WHERE is_immutable=1
              AND node_id NOT IN (SELECT mainplot_target_node_id FROM volumes WHERE mainplot_target_node_id IS NOT NULL)
              AND node_id NOT IN (SELECT mainplot_node_id FROM events WHERE mainplot_node_id IS NOT NULL)
            """
        )
        result = "pass" if not orphans else "fail"
        now = datetime.datetime.utcnow().isoformat()
        db.execute(
            "INSERT INTO finalcheck_signal (check_type, result, detail) VALUES (?,?,?)",
            ("mainplot_orphan_coverage", result, str(orphans) if orphans else None),
        )
        if orphans:
            db.emit_error(f"孤儿主线节点: {orphans}", code=1)
        db.emit_json({"result": "pass"})

    elif args.action == "seal":
        # block if orphan check failed
        bad = db.query(
            "SELECT 1 FROM finalcheck_signal WHERE check_type='mainplot_orphan_coverage' AND result!='pass' LIMIT 1"
        )
        if bad:
            db.emit_error("存在孤儿主线节点，请先通过 finalize check")

        now = datetime.datetime.utcnow().isoformat()
        db.execute(
            "INSERT INTO creation_cycle (status, sealed_at) VALUES ('sealed', ?)", (now,)
        )
        db.emit_json({"ok": True, "sealed_at": now})

    else:
        parser.print_help()
        sys.exit(1)
