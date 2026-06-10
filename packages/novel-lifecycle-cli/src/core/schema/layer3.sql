CREATE TABLE IF NOT EXISTS chapter_writing (
    chapter_id INTEGER PRIMARY KEY REFERENCES chapters(id),
    prose_ref TEXT,
    word_count INTEGER,
    started_at TEXT,
    ended_at TEXT
);

CREATE TABLE IF NOT EXISTS chapter_residue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_id INTEGER NOT NULL REFERENCES chapters(id),
    residue_type TEXT NOT NULL,
    content TEXT NOT NULL,
    resolved INTEGER DEFAULT 0,
    resolved_chapter_id INTEGER,
    escalation_count INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS monitor_signal (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_id INTEGER REFERENCES chapters(id),
    signal_type TEXT NOT NULL,
    detail TEXT,
    created_at TEXT NOT NULL,
    severity TEXT
);

CREATE TABLE IF NOT EXISTS phase_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_id INTEGER NOT NULL REFERENCES chapters(id),
    phase TEXT NOT NULL,
    logged_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS target_convergence_signal (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_id INTEGER NOT NULL REFERENCES chapters(id),
    volume_id INTEGER REFERENCES volumes(id),
    converging INTEGER NOT NULL,
    pulled_by_emergent_id INTEGER REFERENCES emergent(id),
    detail TEXT
);
