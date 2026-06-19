-- Athlete category (M40, SF, U23, …) and explicit anonymous flag.
ALTER TABLE athletes ADD COLUMN category TEXT;
ALTER TABLE athletes ADD COLUMN anonymous INTEGER NOT NULL DEFAULT 0;

-- Rebuild timings to allow the 'DNS' (Did Not Start) status. SQLite cannot
-- ALTER a CHECK constraint, so the table is recreated and copied over.
CREATE TABLE timings_new (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    remote_id           INTEGER UNIQUE,
    athlete_id          INTEGER REFERENCES athletes(id),
    course_id           INTEGER NOT NULL REFERENCES courses(id),
    start_timestamp_ms  INTEGER,
    finish_timestamp_ms INTEGER,
    status              TEXT NOT NULL
                        CHECK(status IN ('Registered','Running','Finished','Withdrawn','DNS')),
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

INSERT INTO timings_new
    SELECT id, remote_id, athlete_id, course_id, start_timestamp_ms,
           finish_timestamp_ms, status, total_time_ms, operator_id,
           duplicate_group_id, duplicate_flagged, created_at_ms, updated_at_ms,
           synced, sync_attempts, last_sync_error
    FROM timings;

DROP TABLE timings;
ALTER TABLE timings_new RENAME TO timings;

CREATE INDEX IF NOT EXISTS idx_timings_athlete ON timings(athlete_id);
CREATE INDEX IF NOT EXISTS idx_timings_course ON timings(course_id);
CREATE INDEX IF NOT EXISTS idx_timings_synced ON timings(synced) WHERE synced = 0;
CREATE INDEX IF NOT EXISTS idx_timings_status ON timings(status);
CREATE INDEX IF NOT EXISTS idx_timings_remote ON timings(remote_id);
CREATE INDEX IF NOT EXISTS idx_timings_dup_group ON timings(duplicate_group_id);

-- Intermediate checkpoints along a course, ordered by position.
CREATE TABLE IF NOT EXISTS checkpoints (
    id            INTEGER PRIMARY KEY,
    course_id     INTEGER NOT NULL REFERENCES courses(id),
    name          TEXT NOT NULL,
    position      INTEGER NOT NULL DEFAULT 0,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_checkpoints_course ON checkpoints(course_id);

-- A split is an athlete's passage time at a checkpoint.
CREATE TABLE IF NOT EXISTS splits (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    remote_id     INTEGER UNIQUE,
    athlete_id    INTEGER NOT NULL REFERENCES athletes(id),
    checkpoint_id INTEGER NOT NULL REFERENCES checkpoints(id),
    course_id     INTEGER NOT NULL REFERENCES courses(id),
    timestamp_ms  INTEGER NOT NULL,
    split_time_ms INTEGER,
    operator_id   TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    synced        INTEGER NOT NULL DEFAULT 0,
    sync_attempts INTEGER NOT NULL DEFAULT 0,
    UNIQUE(athlete_id, checkpoint_id)
);
CREATE INDEX IF NOT EXISTS idx_splits_athlete ON splits(athlete_id);
CREATE INDEX IF NOT EXISTS idx_splits_checkpoint ON splits(checkpoint_id);
CREATE INDEX IF NOT EXISTS idx_splits_course ON splits(course_id);
