use crate::error::AppResult;
use crate::timer::clock::{ClockProvider, SystemClock};
use rusqlite::{params, Connection};
use std::collections::HashSet;

const MIGRATIONS: &[(i32, &str)] = &[
    (1, include_str!("../../migrations/0001_init.sql")),
    (2, include_str!("../../migrations/0002_course_ended_at.sql")),
    (3, include_str!("../../migrations/0003_races.sql")),
];

pub fn run(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        )"
    )?;
    let applied: HashSet<i32> = conn
        .prepare("SELECT version FROM _migrations")?
        .query_map([], |r| r.get::<_, i32>(0))?
        .filter_map(Result::ok).collect();
    let clock = SystemClock;
    for (v, sql) in MIGRATIONS {
        if applied.contains(v) { continue; }
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO _migrations(version, applied_at) VALUES (?, ?)",
            params![v, clock.now_ms()],
        )?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn applies_migrations_to_empty_db() {
        let db = Db::open_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        run(&conn).unwrap();
        let count: i64 = conn.query_row(
            "SELECT count(*) FROM _migrations", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, MIGRATIONS.len() as i64);
        let courses_exists: i64 = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='courses'",
            [], |r| r.get(0)
        ).unwrap();
        assert_eq!(courses_exists, 1);
        let races_exists: i64 = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='races'",
            [], |r| r.get(0)
        ).unwrap();
        assert_eq!(races_exists, 1);
    }

    #[test]
    fn idempotent_when_run_twice() {
        let db = Db::open_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        run(&conn).unwrap();
        run(&conn).unwrap();
        let count: i64 = conn.query_row(
            "SELECT count(*) FROM _migrations", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, MIGRATIONS.len() as i64);
    }
}
