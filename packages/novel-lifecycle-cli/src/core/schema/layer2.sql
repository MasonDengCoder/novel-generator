CREATE TABLE IF NOT EXISTS volumes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    "order" INTEGER,
    mainplot_target_node_id INTEGER REFERENCES mainplot_nodes(node_id),
    target_achieved INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    volume_id INTEGER NOT NULL REFERENCES volumes(id),
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    mainplot_node_id INTEGER REFERENCES mainplot_nodes(node_id),
    is_mainplot_critical INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS sections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL REFERENCES events(id),
    title TEXT NOT NULL,
    mainplot_node_id INTEGER REFERENCES mainplot_nodes(node_id),
    mainplot_must_beat TEXT
);

CREATE TABLE IF NOT EXISTS chapters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    section_id INTEGER NOT NULL REFERENCES sections(id),
    seq INTEGER NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'planned'
);

CREATE TABLE IF NOT EXISTS presummary (
    chapter_id INTEGER PRIMARY KEY REFERENCES chapters(id),
    constraints TEXT NOT NULL DEFAULT '[]',
    scene_config TEXT,
    entry_state TEXT,
    exit_state TEXT,
    emotion_curve TEXT
);

CREATE TABLE IF NOT EXISTS great_cycles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    volume_id INTEGER REFERENCES volumes(id),
    summary TEXT,
    archived_at TEXT,
    target_convergence_verdict TEXT,
    drift_note TEXT
);

CREATE TABLE IF NOT EXISTS emergent (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    type TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    disposition TEXT,
    score REAL,
    impact_scope TEXT,
    created_chapter_id INTEGER,
    score_causal REAL,
    score_character REAL,
    score_theme REAL,
    score_surprise REAL,
    disposition_level TEXT,
    scored_at_chapter_id INTEGER,
    disposition_reason TEXT
);

CREATE TABLE IF NOT EXISTS arc (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    emergent_id INTEGER NOT NULL REFERENCES emergent(id),
    stage TEXT NOT NULL,
    summary TEXT NOT NULL,
    disposition_reason TEXT,
    last_active_chapter_id INTEGER,
    reactivation_flag INTEGER DEFAULT 0,
    is_inferred INTEGER DEFAULT 0,
    inferred_at_chapter_id INTEGER
);

CREATE TABLE IF NOT EXISTS mainplot_decomposition_audit (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mainplot_node_id INTEGER NOT NULL REFERENCES mainplot_nodes(node_id),
    volume_id INTEGER REFERENCES volumes(id),
    event_id INTEGER REFERENCES events(id),
    created_at TEXT NOT NULL
);
