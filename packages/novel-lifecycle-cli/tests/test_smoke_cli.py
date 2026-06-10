"""CLI smoke tests: init idempotent, JSON stdout, gate blocking."""
import json
import os
import sqlite3
import subprocess
import sys
import tempfile

_PKG = os.path.join(os.path.dirname(__file__), "..")
_CLI = os.path.join(_PKG, "bin", "novel-cli")


def run(args, db_path):
    env = {**os.environ, "NOVEL_DB": db_path}
    return subprocess.run(
        [sys.executable, _CLI] + args,
        capture_output=True, text=True, env=env, cwd=_PKG,
    )


def test_init_idempotent():
    with tempfile.TemporaryDirectory() as d:
        db_path = os.path.join(d, "sub", "novel.db")
        r = run(["init", "--db-path", db_path], db_path)
        assert r.returncode == 0, r.stderr
        assert json.loads(r.stdout)["ok"] is True
        # second call — idempotent
        r2 = run(["init", "--db-path", db_path], db_path)
        assert r2.returncode == 0, r2.stderr
        print("test_init_idempotent PASSED")


def test_stage_list_json():
    with tempfile.TemporaryDirectory() as d:
        db_path = os.path.join(d, "novel.db")
        run(["init", "--db-path", db_path], db_path)
        r = run(["stage", "list"], db_path)
        assert r.returncode == 0, r.stderr
        rows = json.loads(r.stdout)
        assert isinstance(rows, list)
        assert any(s["stage_name"] == "inspiration" for s in rows)
        print("test_stage_list_json PASSED")


def test_prose_end_gate_nonzero():
    """prose end with unverified must constraints → non-zero exit."""
    with tempfile.TemporaryDirectory() as d:
        db_path = os.path.join(d, "novel.db")
        run(["init", "--db-path", db_path], db_path)

        conn = sqlite3.connect(db_path)
        conn.execute("INSERT INTO volumes (title) VALUES ('V1')")
        conn.execute("INSERT INTO events (volume_id, title) VALUES (1,'E1')")
        conn.execute("INSERT INTO sections (event_id, title) VALUES (1,'S1')")
        conn.execute("INSERT INTO chapters (section_id, seq, title) VALUES (1,1,'C1')")
        constraints = json.dumps([{"type": "must", "mainplot_node_id": 1, "satisfied": None}])
        conn.execute("INSERT INTO presummary (chapter_id, constraints) VALUES (1,?)", (constraints,))
        conn.execute("INSERT INTO chapter_writing (chapter_id, started_at) VALUES (1,'2026-01-01')")
        conn.commit()
        conn.close()

        r = run(["prose", "end", "1"], db_path)
        assert r.returncode != 0, "expected non-zero exit for missing must verdict"
        print("test_prose_end_gate_nonzero PASSED")


if __name__ == "__main__":
    test_init_idempotent()
    test_stage_list_json()
    test_prose_end_gate_nonzero()
