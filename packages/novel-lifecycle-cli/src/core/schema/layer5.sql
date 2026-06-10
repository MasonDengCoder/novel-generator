CREATE TABLE IF NOT EXISTS chapter_threads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_id INTEGER NOT NULL REFERENCES chapters(id),
    element_type TEXT NOT NULL,
    element_id INTEGER NOT NULL,
    relevance REAL NOT NULL DEFAULT 0.0,
    action TEXT NOT NULL
);
