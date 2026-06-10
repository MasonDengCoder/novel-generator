CREATE TABLE IF NOT EXISTS stage_progress (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stage_name TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'pending',
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS world_boundaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    boundary_type TEXT NOT NULL,
    content TEXT NOT NULL,
    is_fuzzy INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS mainplot_nodes (
    node_id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    is_immutable INTEGER NOT NULL DEFAULT 0,
    "order" INTEGER
);

CREATE TABLE IF NOT EXISTS style_voice (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    aspect TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sensitive_word (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    word TEXT NOT NULL,
    category TEXT,
    severity TEXT
);

CREATE TABLE IF NOT EXISTS subplot (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    silence_chapters INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS foreshadowing (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    planted_at INTEGER,
    resolved_at TEXT,
    resolution_chapter_id INTEGER,
    defer_count INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS payoff (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    foreshadowing_id INTEGER REFERENCES foreshadowing(id),
    content TEXT NOT NULL,
    resolution_chapter_id INTEGER
);

CREATE TABLE IF NOT EXISTS promise (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    deadline_chapter INTEGER,
    violation_flag INTEGER DEFAULT 0
);
