import argparse
import datetime
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_sig = sub.add_parser("signal")
    p_sig.add_argument("--signal-type", required=True)
    p_sig.add_argument("--chapter-id", type=int, default=None)
    p_sig.add_argument("--detail", default=None)
    p_sig.add_argument("--severity", default=None)

    args = parser.parse_args(argv[1:])

    if args.action == "signal":
        now = datetime.datetime.utcnow().isoformat()
        sid = db.execute(
            "INSERT INTO monitor_signal (chapter_id, signal_type, detail, created_at, severity) VALUES (?,?,?,?,?)",
            (args.chapter_id, args.signal_type, args.detail, now, args.severity),
        )
        db.emit_json({"ok": True, "id": sid, "signal_type": args.signal_type})
    else:
        parser.print_help()
        sys.exit(1)
