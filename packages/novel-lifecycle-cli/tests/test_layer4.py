"""Guard: layer4 tables and required columns exist."""
import os, sqlite3, sys, tempfile
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))
from core.db import init_db

_TABLES = {
    "creation_cycle": ["status", "sealed_at"],
    "finalcheck_signal": ["check_type", "result"],
    "creation_log_archive": ["log_type", "content", "archived_at"],
}

def test_layer4():
    with tempfile.NamedTemporaryFile(suffix=".db", delete=False) as f:
        path = f.name
    try:
        init_db(path)
        conn = sqlite3.connect(path)
        for table, cols in _TABLES.items():
            existing = {r[1] for r in conn.execute(f"PRAGMA table_info({table})")}
            assert existing, f"table {table} missing"
            for col in cols:
                assert col in existing, f"{table}.{col} missing"
        conn.close()
        print("test_layer4 PASSED")
    finally:
        os.unlink(path)

if __name__ == "__main__":
    test_layer4()
