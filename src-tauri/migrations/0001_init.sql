PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS courses (
    id              INTEGER PRIMARY KEY,
    name            TEXT NOT NULL,
    distance_m      INTEGER,
    started_at_ms   INTEGER,
    scheduled_at_ms INTEGER,
    created_at_ms   INTEGER NOT NULL,
    updated_at_ms   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS athletes (
    id            INTEGER PRIMARY KEY,
    bib_number    INTEGER NOT NULL UNIQUE,
    first_name    TEXT NOT NULL,
    last_name     TEXT NOT NULL,
    course_id     INTEGER NOT NULL REFERENCES courses(id),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_athletes_bib ON athletes(bib_number);
CREATE INDEX IF NOT EXISTS idx_athletes_course ON athletes(course_id);

CREATE TABLE IF NOT EXISTS timings (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    remote_id           INTEGER UNIQUE,
    athlete_id          INTEGER REFERENCES athletes(id),
    course_id           INTEGER NOT NULL REFERENCES courses(id),
    start_timestamp_ms  INTEGER,
    finish_timestamp_ms INTEGER,
    status              TEXT NOT NULL
                        CHECK(status IN ('Registered','Running','Finished','Withdrawn')),
    total_time_ms       INTEGER,
    operator_id         TEXT NOT NULL,
    duplicate_group_id  TEXT,
    duplicate_flagged   INTEGER NOT NULL DEFAULT 0,
    created_at_ms       INTEGER NOT NULL,
    updated_at_ms       INTEGER NOT NULL,
    synced              INTEGER NOT NULL DEFAULT 0,
    sync_attempts       INTEGER NOT NULL DEFAULT 0,
    last_sync_error     TEXT
);
CREATE INDEX IF NOT EXISTS idx_timings_athlete ON timings(athlete_id);
CREATE INDEX IF NOT EXISTS idx_timings_course ON timings(course_id);
CREATE INDEX IF NOT EXISTS idx_timings_synced ON timings(synced) WHERE synced = 0;
CREATE INDEX IF NOT EXISTS idx_timings_status ON timings(status);
CREATE INDEX IF NOT EXISTS idx_timings_remote ON timings(remote_id);
CREATE INDEX IF NOT EXISTS idx_timings_dup_group ON timings(duplicate_group_id);

CREATE TABLE IF NOT EXISTS pending_finishes (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    remote_id           INTEGER UNIQUE,
    course_id           INTEGER NOT NULL REFERENCES courses(id),
    finish_timestamp_ms INTEGER NOT NULL,
    operator_id         TEXT NOT NULL,
    created_at_ms       INTEGER NOT NULL,
    assigned            INTEGER NOT NULL DEFAULT 0,
    synced              INTEGER NOT NULL DEFAULT 0,
    sync_attempts       INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_pending_open ON pending_finishes(course_id, assigned);

CREATE TABLE IF NOT EXISTS sync_cursor (
    resource             TEXT PRIMARY KEY,
    last_seen_remote_id  INTEGER NOT NULL DEFAULT 0,
    last_pull_at_ms      INTEGER NOT NULL DEFAULT 0
);
