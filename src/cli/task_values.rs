use chrono::{DateTime, NaiveDate, Utc};

use crate::opus_parser::QuickAddParser;

pub fn parse_when(raw: &str) -> Result<Option<DateTime<Utc>>, String> {
    let text = raw.trim();
    if text.eq_ignore_ascii_case("none") || text.eq_ignore_ascii_case("clear") {
        return Ok(None);
    }
    if let Ok(parsed) = DateTime::parse_from_rfc3339(text) {
        return Ok(Some(parsed.with_timezone(&Utc)));
    }
    if let Ok(day) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        return Ok(day.and_hms_opt(0, 0, 0).map(|d| d.and_utc()));
    }
    QuickAddParser::new()
        .parse_date(text)
        .map(Some)
        .ok_or_else(|| format!("cannot read date '{}'", raw))
}

pub fn parse_assignee(raw: &str) -> Option<String> {
    let text = raw.trim();
    if text.eq_ignore_ascii_case("none") || text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn iso_dates_land_on_utc_midnight() {
        let expected = Utc.with_ymd_and_hms(2026, 11, 5, 0, 0, 0).unwrap();
        assert_eq!(parse_when("2026-11-05"), Ok(Some(expected)));
    }

    #[test]
    fn rfc3339_keeps_the_instant() {
        let expected = Utc.with_ymd_and_hms(2026, 11, 5, 8, 30, 0).unwrap();
        assert_eq!(parse_when("2026-11-05T09:30:00+01:00"), Ok(Some(expected)));
    }

    #[test]
    fn none_and_clear_mean_no_date() {
        assert_eq!(parse_when("none"), Ok(None));
        assert_eq!(parse_when("CLEAR"), Ok(None));
    }

    #[test]
    fn garbage_dates_are_rejected() {
        assert!(parse_when("whenever-ish").is_err());
    }
}
