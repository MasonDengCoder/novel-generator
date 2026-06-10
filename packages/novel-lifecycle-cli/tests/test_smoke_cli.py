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


def test_full_chain_end_to_end():
    """奠基录入 → finalcheck → stage → weave → prose 遥测 → convergence → close → finalize."""
    with tempfile.TemporaryDirectory() as d:
        db_path = os.path.join(d, "novel.db")
        run(["init", "--db-path", db_path], db_path)

        def ok(args):
            r = run(args, db_path)
            assert r.returncode == 0, f"{args} failed: {r.stderr}"
            return r

        # foundation
        ok(["mainplot", "add-node", "--content", "终局", "--immutable", "1", "--order", "1"])
        ok(["world-boundary", "add", "--type", "hard", "--content", "无复活"])
        ok(["finalcheck", "signal", "--check-type", "boundary_validated", "--result", "pass"])
        # stage gate now passes
        ok(["stage", "complete", "world-boundary"])
        # weave structure
        ok(["weave", "volume", "--title", "V1", "--mainplot-target-node-id", "1"])
        ok(["weave", "event", "--volume-id", "1", "--title", "E1",
            "--mainplot-node-id", "1", "--is-mainplot-critical", "1"])
        ok(["weave", "section", "--event-id", "1", "--title", "S1"])
        ok(["weave", "chapter", "--section-id", "1", "--seq", "1", "--title", "C1"])
        # presummary with satisfied must
        ok(["presummary", "set", "--chapter-id", "1",
            "--constraints", json.dumps([{"type": "must", "mainplot_node_id": 1, "satisfied": 1}])])
        # prose with real telemetry
        ok(["prose", "start", "1", "--prose-ref", "manuscripts/c1.md"])
        r = ok(["prose", "end", "1", "--prose-ref", "manuscripts/c1.md", "--word-count", "3200"])
        assert json.loads(r.stdout)["word_count"] == 3200
        # convergence
        ok(["cycle", "converge-signal", "--chapter-id", "1", "--volume-id", "1", "--converging", "1"])
        ok(["cycle", "convergence-verdict", "--volume-id", "1",
            "--verdict", "achieved", "--target-achieved", "1"])
        # close event + volume
        ok(["cycle", "close-event", "1"])
        ok(["cycle", "close-volume", "1"])
        # finalize
        ok(["finalize", "check"])
        ok(["finalize", "seal"])
        print("test_full_chain_end_to_end PASSED")


def test_convergence_verdict_upsert():
    """同卷重复 convergence-verdict 不产生第二行 great_cycles."""
    with tempfile.TemporaryDirectory() as d:
        db_path = os.path.join(d, "novel.db")
        run(["init", "--db-path", db_path], db_path)
        run(["weave", "volume", "--title", "V1"], db_path)
        run(["cycle", "convergence-verdict", "--volume-id", "1", "--verdict", "a"], db_path)
        run(["cycle", "convergence-verdict", "--volume-id", "1", "--verdict", "b"], db_path)
        conn = sqlite3.connect(db_path)
        n = conn.execute("SELECT COUNT(*) FROM great_cycles WHERE volume_id=1").fetchone()[0]
        verdict = conn.execute(
            "SELECT target_convergence_verdict FROM great_cycles WHERE volume_id=1").fetchone()[0]
        conn.close()
        assert n == 1, f"expected 1 great_cycles row, got {n}"
        assert verdict == "b", f"expected latest verdict 'b', got {verdict}"
        print("test_convergence_verdict_upsert PASSED")


def test_seal_blocked_without_check():
    """未跑 finalize check → seal 阻断；db get 非白名单表名 → 拒绝."""
    with tempfile.TemporaryDirectory() as d:
        db_path = os.path.join(d, "novel.db")
        run(["init", "--db-path", db_path], db_path)
        r = run(["finalize", "seal"], db_path)
        assert r.returncode != 0, "expected seal blocked without passing check"
        r2 = run(["db", "get", "--type", "users", "--id", "1"], db_path)
        assert r2.returncode != 0, "expected db get to reject non-whitelist table"
        print("test_seal_blocked_without_check PASSED")


if __name__ == "__main__":
    test_init_idempotent()
    test_stage_list_json()
    test_prose_end_gate_nonzero()
    test_full_chain_end_to_end()
    test_convergence_verdict_upsert()
    test_seal_blocked_without_check()
