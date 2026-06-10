CREATE TABLE IF NOT EXISTS creation_cycle (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    status TEXT NOT NULL DEFAULT 'active',
    sealed_at TEXT
);

CREATE TABLE IF NOT EXISTS finalcheck_signal (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cycle_id INTEGER REFERENCES creation_cycle(id),
    check_type TEXT NOT NULL,
    result TEXT NOT NULL,
    detail TEXT
);

CREATE TABLE IF NOT EXISTS creation_log_archive (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cycle_id INTEGER REFERENCES creation_cycle(id),
    log_type TEXT NOT NULL,
    content TEXT NOT NULL,
    archived_at TEXT NOT NULL
);
