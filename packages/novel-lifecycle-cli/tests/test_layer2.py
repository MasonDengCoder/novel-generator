"""Guard: layer2 tables and required columns exist."""
import os, sqlite3, sys, tempfile
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))
from core.db import init_db

_TABLES = {
    "volumes": ["title", "status", "mainplot_target_node_id", "target_achieved"],
    "events": ["volume_id", "title", "status", "mainplot_node_id", "is_mainplot_critical"],
    "sections": ["event_id", "title", "mainplot_must_beat"],
    "chapters": ["section_id", "seq", "title", "status"],
    "presummary": ["chapter_id", "constraints"],
    "great_cycles": ["volume_id", "target_convergence_verdict"],
    "emergent": ["type", "name", "status", "score_causal", "score_character", "score_theme", "score_surprise"],
    "arc": ["emergent_id", "stage", "summary", "reactivation_flag", "is_inferred"],
    "mainplot_decomposition_audit": ["mainplot_node_id", "created_at"],
}

def test_layer2():
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
        print("test_layer2 PASSED")
    finally:
        os.unlink(path)

if __name__ == "__main__":
    test_layer2()
