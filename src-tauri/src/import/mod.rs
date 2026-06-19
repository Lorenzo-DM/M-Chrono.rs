use crate::db::repo::Repo;
use crate::error::{AppError, AppResult};
use crate::models::{Athlete, Course};
use calamine::{open_workbook_auto, Data, Reader};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ImportRowError {
    pub row: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportSummary {
    pub inserted: usize,
    pub updated: usize,
    pub courses_created: usize,
    pub errors: Vec<ImportRowError>,
}

#[derive(Debug, Clone)]
pub struct RawRow {
    pub row_num: usize,
    pub bib: i64,
    pub first_name: String,
    pub last_name: String,
    pub course_name: String,
    pub category: Option<String>,
}

/// Expected columns, in order: pettorale | nome | cognome | percorso | categoria.
/// The category column is optional.
/// A header row is auto-skipped when the first cell of row 1 is not an integer.
pub fn import_file(repo: &Repo, path: &Path) -> AppResult<ImportSummary> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    let (rows, mut errors) = match ext.as_str() {
        "xlsx" | "xls" | "ods" => parse_xlsx(path)?,
        "csv" | "txt" => parse_csv(path)?,
        other => {
            return Err(AppError::InvalidState(format!(
                "formato file non supportato: .{other} (usa .xlsx o .csv)"
            )))
        }
    };
    let mut summary = apply_rows(repo, rows)?;
    errors.append(&mut summary.errors);
    errors.sort_by_key(|e| e.row);
    summary.errors = errors;
    Ok(summary)
}

pub fn parse_xlsx(path: &Path) -> AppResult<(Vec<RawRow>, Vec<ImportRowError>)> {
    let mut wb = open_workbook_auto(path)
        .map_err(|e| AppError::InvalidState(format!("impossibile aprire il file: {e}")))?;
    let range = wb
        .worksheet_range_at(0)
        .ok_or_else(|| AppError::InvalidState("il file non contiene fogli".into()))?
        .map_err(|e| AppError::InvalidState(format!("impossibile leggere il foglio: {e}")))?;

    let mut rows = Vec::new();
    let mut errors = Vec::new();
    for (i, cells) in range.rows().enumerate() {
        let row_num = i + 1;
        let bib_raw = cells.first().map(cell_to_string).unwrap_or_default();
        if row_num == 1 && bib_raw.trim().parse::<i64>().is_err() {
            continue; // header row
        }
        if cells.iter().all(|c| matches!(c, Data::Empty)) {
            continue;
        }
        let first = cells.get(1).map(cell_to_string).unwrap_or_default();
        let last = cells.get(2).map(cell_to_string).unwrap_or_default();
        let course = cells.get(3).map(cell_to_string).unwrap_or_default();
        let category = cells.get(4).map(cell_to_string).unwrap_or_default();
        match build_row(row_num, &bib_raw, &first, &last, &course, &category) {
            Ok(r) => rows.push(r),
            Err(e) => errors.push(e),
        }
    }
    Ok((rows, errors))
}

pub fn parse_csv(path: &Path) -> AppResult<(Vec<RawRow>, Vec<ImportRowError>)> {
    let bytes = std::fs::read(path)?;
    let text = String::from_utf8_lossy(&bytes);
    // Italian Excel exports CSV with ';' — pick the delimiter that appears most in line 1.
    let first_line = text.lines().next().unwrap_or("");
    let delim = if first_line.matches(';').count() >= first_line.matches(',').count() {
        b';'
    } else {
        b','
    };

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut rows = Vec::new();
    let mut errors = Vec::new();
    for (i, rec) in rdr.records().enumerate() {
        let row_num = i + 1;
        let rec = match rec {
            Ok(r) => r,
            Err(e) => {
                errors.push(ImportRowError { row: row_num, message: format!("riga illeggibile: {e}") });
                continue;
            }
        };
        let bib_raw = rec.get(0).unwrap_or("").to_string();
        if row_num == 1 && bib_raw.trim().parse::<i64>().is_err() {
            continue; // header row
        }
        if rec.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let first = rec.get(1).unwrap_or("").to_string();
        let last = rec.get(2).unwrap_or("").to_string();
        let course = rec.get(3).unwrap_or("").to_string();
        let category = rec.get(4).unwrap_or("").to_string();
        match build_row(row_num, &bib_raw, &first, &last, &course, &category) {
            Ok(r) => rows.push(r),
            Err(e) => errors.push(e),
        }
    }
    Ok((rows, errors))
}

fn build_row(
    row_num: usize,
    bib_raw: &str,
    first: &str,
    last: &str,
    course: &str,
    category: &str,
) -> Result<RawRow, ImportRowError> {
    let bib = bib_raw.trim().parse::<i64>().map_err(|_| ImportRowError {
        row: row_num,
        message: format!("pettorale non valido: \"{}\"", bib_raw.trim()),
    })?;
    if bib <= 0 {
        return Err(ImportRowError {
            row: row_num,
            message: format!("pettorale deve essere positivo: {bib}"),
        });
    }
    let first_name = first.trim().to_string();
    let last_name = last.trim().to_string();
    if first_name.is_empty() && last_name.is_empty() {
        return Err(ImportRowError { row: row_num, message: "nome e cognome mancanti".into() });
    }
    let course_name = course.trim().to_string();
    if course_name.is_empty() {
        return Err(ImportRowError { row: row_num, message: "percorso mancante".into() });
    }
    let category = {
        let c = category.trim();
        if c.is_empty() { None } else { Some(c.to_string()) }
    };
    Ok(RawRow { row_num, bib, first_name, last_name, course_name, category })
}

fn cell_to_string(c: &Data) -> String {
    match c {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Data::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

/// Resolve a course by trimmed, case-insensitive name; create it locally
/// (negative id) when missing. Returns (course_id, created).
pub fn get_or_create_course(repo: &Repo, name: &str) -> AppResult<(i64, bool)> {
    if let Some(c) = repo.find_course_by_name(name)? {
        return Ok((c.id, false));
    }
    let id = repo.next_local_course_id()?;
    repo.upsert_course(&Course {
        id,
        name: name.trim().to_string(),
        distance_m: None,
        started_at_ms: None,
        scheduled_at_ms: None,
        ended_at_ms: None, race_id: None,
    })?;
    Ok((id, true))
}

/// Upsert-by-bib: an existing bib is updated in place (keeps its id, even a
/// backend-positive one); a new bib is inserted with the next local negative id.
/// Duplicate bibs within the same batch: first occurrence wins.
pub fn apply_rows(repo: &Repo, rows: Vec<RawRow>) -> AppResult<ImportSummary> {
    let mut summary = ImportSummary {
        inserted: 0,
        updated: 0,
        courses_created: 0,
        errors: Vec::new(),
    };
    let mut course_ids: HashMap<String, i64> = HashMap::new();
    let mut seen_bibs: HashSet<i64> = HashSet::new();
    let mut next_athlete_id = repo.next_local_athlete_id()?;

    for r in rows {
        if !seen_bibs.insert(r.bib) {
            summary.errors.push(ImportRowError {
                row: r.row_num,
                message: format!("pettorale {} duplicato nel file", r.bib),
            });
            continue;
        }
        let key = r.course_name.trim().to_lowercase();
        let course_id = match course_ids.get(&key) {
            Some(id) => *id,
            None => {
                let (id, created) = get_or_create_course(repo, &r.course_name)?;
                if created {
                    summary.courses_created += 1;
                }
                course_ids.insert(key, id);
                id
            }
        };
        match repo.find_athlete_by_bib(r.bib)? {
            Some(existing) => {
                repo.upsert_athlete(&Athlete {
                    id: existing.id,
                    bib_number: r.bib,
                    first_name: r.first_name,
                    last_name: r.last_name,
                    course_id,
                    category: r.category.or(existing.category),
                    anonymous: existing.anonymous,
                })?;
                summary.updated += 1;
            }
            None => {
                repo.upsert_athlete(&Athlete {
                    id: next_athlete_id,
                    bib_number: r.bib,
                    first_name: r.first_name,
                    last_name: r.last_name,
                    course_id,
                    category: r.category,
                    anonymous: false,
                })?;
                next_athlete_id -= 1;
                summary.inserted += 1;
            }
        }
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrations, Db};

    fn fresh() -> Repo {
        let db = Db::open_in_memory().unwrap();
        migrations::run(&db.conn.lock().unwrap()).unwrap();
        Repo::new(db.conn.clone())
    }

    fn row(n: usize, bib: i64, first: &str, last: &str, course: &str) -> RawRow {
        RawRow {
            row_num: n,
            bib,
            first_name: first.into(),
            last_name: last.into(),
            course_name: course.into(),
            category: None,
        }
    }

    #[test]
    fn apply_creates_local_courses_with_negative_ids() {
        let r = fresh();
        let s = apply_rows(&r, vec![
            row(1, 1, "Mario", "Rossi", "21K"),
            row(2, 2, "Anna", "Bianchi", "10K"),
        ]).unwrap();
        assert_eq!(s.inserted, 2);
        assert_eq!(s.courses_created, 2);
        let courses = r.list_courses().unwrap();
        assert!(courses.iter().all(|c| c.id < 0));
        let athletes = r.list_athletes().unwrap();
        assert!(athletes.iter().all(|a| a.id < 0));
    }

    #[test]
    fn apply_matches_existing_course_case_insensitive() {
        let r = fresh();
        r.upsert_course(&Course {
            id: 5, name: "21K".into(), distance_m: None,
            started_at_ms: None, scheduled_at_ms: None, ended_at_ms: None, race_id: None,
        }).unwrap();
        let s = apply_rows(&r, vec![row(1, 1, "Mario", "Rossi", "  21k ")]).unwrap();
        assert_eq!(s.courses_created, 0);
        assert_eq!(r.list_athletes().unwrap()[0].course_id, 5);
    }

    #[test]
    fn apply_upserts_by_bib_keeping_id() {
        let r = fresh();
        r.upsert_course(&Course {
            id: 5, name: "21K".into(), distance_m: None,
            started_at_ms: None, scheduled_at_ms: None, ended_at_ms: None, race_id: None,
        }).unwrap();
        r.upsert_athlete(&Athlete {
            id: 100, bib_number: 7, first_name: "Old".into(),
            last_name: "Name".into(), course_id: 5, category: None, anonymous: false,
        }).unwrap();
        let s = apply_rows(&r, vec![row(1, 7, "Mario", "Rossi", "21K")]).unwrap();
        assert_eq!(s.updated, 1);
        assert_eq!(s.inserted, 0);
        let a = r.find_athlete_by_bib(7).unwrap().unwrap();
        assert_eq!(a.id, 100);
        assert_eq!(a.first_name, "Mario");
    }

    #[test]
    fn apply_rejects_duplicate_bib_in_batch() {
        let r = fresh();
        let s = apply_rows(&r, vec![
            row(1, 7, "Mario", "Rossi", "21K"),
            row(2, 7, "Anna", "Bianchi", "21K"),
        ]).unwrap();
        assert_eq!(s.inserted, 1);
        assert_eq!(s.errors.len(), 1);
        assert_eq!(s.errors[0].row, 2);
        let a = r.find_athlete_by_bib(7).unwrap().unwrap();
        assert_eq!(a.first_name, "Mario");
    }

    #[test]
    fn reimport_is_idempotent() {
        let r = fresh();
        let rows = vec![row(1, 1, "Mario", "Rossi", "21K"), row(2, 2, "Anna", "Bianchi", "21K")];
        let s1 = apply_rows(&r, rows.clone()).unwrap();
        assert_eq!(s1.inserted, 2);
        let s2 = apply_rows(&r, rows).unwrap();
        assert_eq!(s2.inserted, 0);
        assert_eq!(s2.updated, 2);
        assert_eq!(r.list_athletes().unwrap().len(), 2);
    }

    #[test]
    fn build_row_validates() {
        assert!(build_row(1, "abc", "a", "b", "21K", "").is_err());
        assert!(build_row(1, "-3", "a", "b", "21K", "").is_err());
        assert!(build_row(1, "5", "", "", "21K", "").is_err());
        assert!(build_row(1, "5", "a", "b", " ", "").is_err());
        assert!(build_row(1, " 5 ", "a", "", "21K", "").is_ok());
        assert_eq!(build_row(1, "5", "a", "b", "21K", " M40 ").unwrap().category.as_deref(), Some("M40"));
    }

    #[test]
    fn csv_delimiter_autodetect_and_header_skip() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.csv");
        std::fs::write(&p, "pettorale;nome;cognome;percorso\n1;Mario;Rossi;21K\n2;Anna;Bianchi;10K\n").unwrap();
        let (rows, errors) = parse_csv(&p).unwrap();
        assert!(errors.is_empty());
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].bib, 1);
        assert_eq!(rows[1].course_name, "10K");
    }

    #[test]
    fn csv_collects_row_errors_and_continues() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.csv");
        std::fs::write(&p, "1,Mario,Rossi,21K\nxx,Anna,Bianchi,10K\n3,Luca,Verdi,10K\n").unwrap();
        let (rows, errors) = parse_csv(&p).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].row, 2);
    }
}
