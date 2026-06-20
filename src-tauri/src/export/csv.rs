use crate::db::repo::Repo;
use crate::error::{AppError, AppResult};
use crate::export::{fmt_clock, fmt_hms};
use std::path::Path;

#[derive(Debug, serde::Serialize)]
pub struct ExportSummary {
    pub path: String,
    pub courses_count: usize,
    pub athletes_count: usize,
}

const HEADERS: [&str; 10] = [
    "Pos", "Pettorale", "Cognome", "Nome", "Categoria", "Percorso",
    "Arrivo", "Tempo", "Status", "Flag",
];

/// Write all results to a single CSV (`;`-delimited for Italian Excel),
/// grouped per course with a position column reset per course.
pub fn write_results(repo: &Repo, path: &Path) -> AppResult<ExportSummary> {
    let courses = repo.list_courses()?;
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_path(path)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    wtr.write_record(HEADERS).map_err(|e| AppError::Internal(e.to_string()))?;

    let mut total = 0usize;
    for course in &courses {
        for (idx, r) in repo.list_results_by_course(course.id)?.iter().enumerate() {
            total += 1;
            wtr.write_record([
                (idx + 1).to_string(),
                r.bib_number.map(|b| b.to_string()).unwrap_or_default(),
                r.last_name.clone().unwrap_or_default(),
                r.first_name.clone().unwrap_or_default(),
                r.category.clone().unwrap_or_default(),
                r.course_name.clone(),
                fmt_clock(r.finish_timestamp_ms),
                fmt_hms(r.total_time_ms),
                r.status.clone(),
                if r.duplicate_flagged { "DUP".into() } else { String::new() },
            ])
            .map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }
    wtr.flush().map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(ExportSummary {
        path: path.to_string_lossy().into(),
        courses_count: courses.len(),
        athletes_count: total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrations, Db};
    use crate::models::{Athlete, Course};
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn writes_csv_with_header_and_rows() {
        let db = Db::open_in_memory().unwrap();
        migrations::run(&db.conn.lock().unwrap()).unwrap();
        let repo = Arc::new(Repo::new(db.conn.clone()));
        repo.upsert_course(&Course {
            id: 1, name: "21K".into(), distance_m: None, started_at_ms: Some(0),
            scheduled_at_ms: None, ended_at_ms: None, race_id: None,
        }).unwrap();
        repo.upsert_athlete(&Athlete {
            id: 1, bib_number: 7, first_name: "M".into(), last_name: "R".into(),
            course_id: 1, category: Some("M40".into()), anonymous: false,
        }).unwrap();
        let tid = repo.insert_timing_running(1, 1, 0, "PC-A").unwrap();
        repo.update_finish(tid, 1000, 1000).unwrap();

        let dir = tempdir().unwrap();
        let path = dir.path().join("out.csv");
        let s = write_results(&repo, &path).unwrap();
        assert_eq!(s.athletes_count, 1);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("Categoria"));
        assert!(text.contains("M40"));
        assert!(text.contains("00:00:01.000"));
    }
}
