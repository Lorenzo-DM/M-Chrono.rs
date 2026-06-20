pub mod xlsx;
pub mod csv;

/// Format a millisecond duration as HH:MM:SS.mmm. Returns empty string for None.
pub fn fmt_hms(ms: Option<i64>) -> String {
    match ms {
        Some(ms) if ms >= 0 => {
            let millis = ms % 1000;
            let total_sec = ms / 1000;
            let sec = total_sec % 60;
            let min = (total_sec / 60) % 60;
            let hrs = total_sec / 3600;
            format!("{hrs:02}:{min:02}:{sec:02}.{millis:03}")
        }
        _ => String::new(),
    }
}

/// Format an absolute epoch-ms timestamp as a local-clock wall time HH:MM:SS.mmm.
pub fn fmt_clock(ms: Option<i64>) -> String {
    match ms {
        Some(ms) if ms >= 0 => fmt_hms(Some(ms % 86_400_000)),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_hms_formats_and_handles_none() {
        assert_eq!(fmt_hms(Some(3_661_500)), "01:01:01.500");
        assert_eq!(fmt_hms(Some(0)), "00:00:00.000");
        assert_eq!(fmt_hms(None), "");
        assert_eq!(fmt_hms(Some(-5)), "");
    }
}
