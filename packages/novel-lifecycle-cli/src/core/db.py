import json
import os
import sqlite3
import sys

_LAYERS = ["layer1.sql", "layer2.sql", "layer3.sql", "layer4.sql", "layer5.sql"]
_SCHEMA_DIR = os.path.join(os.path.dirname(__file__), "schema")

_db_path = None


def init_db(db_path):
    global _db_path
    _db_path = db_path
    os.makedirs(os.path.dirname(os.path.abspath(db_path)), exist_ok=True)
    conn = sqlite3.connect(db_path)
    for layer in _LAYERS:
        sql = open(os.path.join(_SCHEMA_DIR, layer)).read()
        conn.executescript(sql)
    conn.commit()
    conn.close()


def get_conn():
    if not _db_path:
        raise RuntimeError("DB not initialized; call init_db() first")
    return sqlite3.connect(_db_path)


def _is_sealed(conn):
    try:
        row = conn.execute(
            "SELECT 1 FROM creation_cycle WHERE status='sealed' LIMIT 1"
        ).fetchone()
        return row is not None
    except sqlite3.OperationalError:
        return False


def execute(sql, params=()):
    conn = get_conn()
    sql_upper = sql.strip().upper()
    write_ops = ("INSERT", "UPDATE", "DELETE", "DROP", "ALTER", "CREATE")
    if any(sql_upper.startswith(op) for op in write_ops):
        if _is_sealed(conn):
            conn.close()
            emit_error("DB 已封档，拒绝写入", code=3)
    try:
        cur = conn.execute(sql, params)
        conn.commit()
        last_id = cur.lastrowid
        conn.close()
        return last_id
    except Exception as e:
        conn.close()
        raise


def query(sql, params=()):
    conn = get_conn()
    try:
        cur = conn.execute(sql, params)
        cols = [d[0] for d in cur.description]
        rows = cur.fetchall()
        conn.close()
        return [dict(zip(cols, row)) for row in rows]
    except Exception as e:
        conn.close()
        raise


def emit_json(data):
    print(json.dumps(data, ensure_ascii=False))


def emit_error(msg, code=1):
    print(msg, file=sys.stderr)
    sys.exit(code)
