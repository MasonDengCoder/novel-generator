import argparse
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="view")
    sub.add_parser("core")
    sub.add_parser("active")
    sub.add_parser("dormant")
    sub.add_parser("full")
    args = parser.parse_args(argv[1:])

    if args.view == "core":
        db.emit_json({
            "world_boundaries": db.query("SELECT * FROM world_boundaries"),
            "mainplot_nodes": db.query("SELECT * FROM mainplot_nodes WHERE is_immutable=1"),
            "style_voice": db.query("SELECT * FROM style_voice"),
        })
    elif args.view == "active":
        db.emit_json({
            "subplots": db.query("SELECT * FROM subplot WHERE status='active'"),
            "emergent": db.query("SELECT * FROM emergent WHERE status='pending'"),
        })
    elif args.view == "dormant":
        db.emit_json({
            "subplots": db.query("SELECT * FROM subplot WHERE status!='active'"),
            "emergent": db.query("SELECT * FROM emergent WHERE status NOT IN ('pending')"),
        })
    elif args.view == "full":
        core = {
            "world_boundaries": db.query("SELECT * FROM world_boundaries"),
            "mainplot_nodes": db.query("SELECT * FROM mainplot_nodes"),
            "style_voice": db.query("SELECT * FROM style_voice"),
        }
        active = {
            "subplots": db.query("SELECT * FROM subplot WHERE status='active'"),
            "emergent": db.query("SELECT * FROM emergent WHERE status='pending'"),
        }
        dormant = {
            "subplots": db.query("SELECT * FROM subplot WHERE status!='active'"),
        }
        db.emit_json({**core, **active, "dormant": dormant})
    else:
        parser.print_help()
        sys.exit(1)
