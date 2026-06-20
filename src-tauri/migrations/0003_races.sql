CREATE TABLE IF NOT EXISTS races (
    id              INTEGER PRIMARY KEY,
    name            TEXT NOT NULL,
    scheduled_at_ms INTEGER,
    created_at_ms   INTEGER NOT NULL,
    updated_at_ms   INTEGER NOT NULL
);

ALTER TABLE courses ADD COLUMN race_id INTEGER REFERENCES races(id);
CREATE INDEX IF NOT EXISTS idx_courses_race ON courses(race_id);
