import argparse
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action")

    p_sig = sub.add_parser("signal")
    p_sig.add_argument("--check-type", required=True)
    p_sig.add_argument("--result", required=True)
    p_sig.add_argument("--detail", default=None)

    args = parser.parse_args(argv[1:])

    if args.action == "signal":
        sid = db.execute(
            "INSERT INTO finalcheck_signal (check_type, result, detail) VALUES (?,?,?)",
            (args.check_type, args.result, args.detail),
        )
        db.emit_json({"ok": True, "id": sid, "check_type": args.check_type, "result": args.result})
    else:
        parser.print_help()
        sys.exit(1)
