"""Guard: 6 load-bearing fields must exist with correct SQLite types."""
import json
import os
import sqlite3
import sys
import tempfile

# Put src on path so we can import db
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))
from core.db import init_db


def _cols(conn, table):
    return {r[1]: r[2].upper() for r in conn.execute(f"PRAGMA table_info({table})")}


def test_load_bearing_fields():
    with tempfile.NamedTemporaryFile(suffix=".db", delete=False) as f:
        path = f.name
    try:
        init_db(path)
        conn = sqlite3.connect(path)

        # presummary.constraints — TEXT (JSON array holding satisfied/mainplot_node_id)
        cols = _cols(conn, "presummary")
        assert "constraints" in cols, "presummary.constraints missing"
        assert cols["constraints"] == "TEXT", f"presummary.constraints type: {cols['constraints']}"

        # events.is_mainplot_critical — INTEGER
        cols = _cols(conn, "events")
        assert "is_mainplot_critical" in cols, "events.is_mainplot_critical missing"
        assert cols["is_mainplot_critical"] == "INTEGER", f"events.is_mainplot_critical type: {cols['is_mainplot_critical']}"

        # volumes.target_achieved — INTEGER
        # volumes.mainplot_target_node_id — INTEGER
        cols = _cols(conn, "volumes")
        assert "target_achieved" in cols, "volumes.target_achieved missing"
        assert cols["target_achieved"] == "INTEGER", f"volumes.target_achieved type: {cols['target_achieved']}"
        assert "mainplot_target_node_id" in cols, "volumes.mainplot_target_node_id missing"
        assert cols["mainplot_target_node_id"] == "INTEGER", f"volumes.mainplot_target_node_id type: {cols['mainplot_target_node_id']}"

        # target_convergence_signal.converging — INTEGER
        cols = _cols(conn, "target_convergence_signal")
        assert "converging" in cols, "target_convergence_signal.converging missing"
        assert cols["converging"] == "INTEGER", f"target_convergence_signal.converging type: {cols['converging']}"

        conn.close()
        print("test_load_bearing_fields PASSED")
    finally:
        os.unlink(path)


if __name__ == "__main__":
    test_load_bearing_fields()
