import argparse
import os
import sys
from core import db


def main(argv):
    parser = argparse.ArgumentParser()
    parser.add_argument("--db-path", default="index/novel.db")
    args = parser.parse_args(argv[1:])
    db.init_db(args.db_path)
    db.emit_json({"ok": True, "db_path": args.db_path})
