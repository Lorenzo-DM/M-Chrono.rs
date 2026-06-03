use crate::db::repo::{Repo, ResultRow};
use crate::error::{AppError, AppResult};
use rust_xlsxwriter::{Color, Format, Workbook, Worksheet};
use std::path::Path;

#[derive(Debug, serde::Serialize)]
pub struct ExportSummary {
    pub path: String,
    pub courses_count: usize,
    pub athletes_count: usize,
}

const HEADERS: [&str; 11] = [
    "Pos",
    "Pettorale",
    "Cognome",
    "Nome",
    "Percorso",
    "Start (ms)",
    "Finish (ms)",
    "Totale (ms)",
    "Status",
    "Operatore",
    "Flag",
];

pub fn write_results(repo: &Repo, path: &Path) -> AppResult<ExportSummary> {
    let mut wb = Workbook::new();
    let header_fmt = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xDDDDDD));
    let dup_fmt = Format::new().set_background_color(Color::RGB(0xFFFF99));

    let courses = repo.list_courses()?;
    let mut total_athletes = 0usize;

    // Summary sheet — collect all rows first, then write
    let mut all_rows: Vec<(usize, ResultRow)> = Vec::new();
    for course in &courses {
        let rows = repo.list_results_by_course(course.id)?;
        for r in rows {
            total_athletes += 1;
            all_rows.push((total_athletes, r));
        }
    }

    {
        let summary = wb.add_worksheet();
        summary
            .set_name("Riepilogo")
            .map_err(|e| AppError::Internal(e.to_string()))?;
        write_header_row(summary, &header_fmt)?;
        for (pos, r) in &all_rows {
            write_data_row(summary, *pos as u32, *pos, r, &dup_fmt)?;
        }
        summary.autofit();
    }

    // Per-course sheets
    for course in &courses {
        let rows = repo.list_results_by_course(course.id)?;
        let ws = wb.add_worksheet();
        ws.set_name(&course.name)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        write_header_row(ws, &header_fmt)?;
        for (idx, r) in rows.iter().enumerate() {
            write_data_row(ws, (idx + 1) as u32, idx + 1, r, &dup_fmt)?;
        }
        ws.autofit();
    }

    wb.save(path)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(ExportSummary {
        path: path.to_string_lossy().into(),
        courses_count: courses.len(),
        athletes_count: total_athletes,
    })
}

fn write_header_row(ws: &mut Worksheet, fmt: &Format) -> AppResult<()> {
    for (i, h) in HEADERS.iter().enumerate() {
        ws.write_with_format(0, i as u16, *h, fmt)
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(())
}

fn write_data_row(
    ws: &mut Worksheet,
    row: u32,
    pos: usize,
    r: &ResultRow,
    dup_fmt: &Format,
) -> AppResult<()> {
    let flag_str = if r.duplicate_flagged {
        "DUP".to_string()
    } else {
        String::new()
    };

    macro_rules! wcell {
        ($col:expr, $val:expr) => {
            if r.duplicate_flagged {
                ws.write_with_format(row, $col, $val, dup_fmt)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
            } else {
                ws.write(row, $col, $val)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
            }
        };
    }

    wcell!(0u16, pos as i64);
    wcell!(1u16, r.bib_number.unwrap_or(0));
    wcell!(2u16, r.last_name.as_deref().unwrap_or(""));
    wcell!(3u16, r.first_name.as_deref().unwrap_or(""));
    wcell!(4u16, r.course_name.as_str());
    wcell!(5u16, r.start_timestamp_ms.unwrap_or(0));
    wcell!(6u16, r.finish_timestamp_ms.unwrap_or(0));
    wcell!(7u16, r.total_time_ms.unwrap_or(0));
    wcell!(8u16, r.status.as_str());
    wcell!(9u16, r.operator_id.as_str());
    wcell!(10u16, flag_str.as_str());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrations, Db};
    use crate::models::{Athlete, Course};
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn writes_xlsx_with_per_course_sheets() {
        let db = Db::open_in_memory().unwrap();
        migrations::run(&db.conn.lock().unwrap()).unwrap();
        let repo = Arc::new(Repo::new(db.conn.clone()));
        repo.upsert_course(&Course {
            id: 1,
            name: "21K".into(),
            distance_m: None,
            started_at_ms: None,
            scheduled_at_ms: None,
            ended_at_ms: None,
        })
        .unwrap();
        repo.upsert_athlete(&Athlete {
            id: 1,
            bib_number: 7,
            first_name: "M".into(),
            last_name: "R".into(),
            course_id: 1,
        })
        .unwrap();
        let tid = repo
            .insert_timing_running(1, 1, 0, "PC-A")
            .unwrap();
        repo.update_finish(tid, 1000, 1000).unwrap();

        let dir = tempdir().unwrap();
        let path = dir.path().join("out.xlsx");
        let summary = write_results(&repo, &path).unwrap();
        assert_eq!(summary.courses_count, 1);
        assert_eq!(summary.athletes_count, 1);
        assert!(path.exists());
        let size = std::fs::metadata(&path).unwrap().len();
        assert!(size > 1000, "xlsx too small: {size}");
    }
}
