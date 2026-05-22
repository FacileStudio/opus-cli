use crate::debug::debug_log;
use aho_corasick::AhoCorasick;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Utc};
use chrono_english::{parse_date_string, Dialect};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct ParsedTask {
    pub title: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub start_date: Option<DateTime<Utc>>,
    pub repeat_interval: Option<RepeatInterval>,
}

#[derive(Debug, Clone)]
pub struct RepeatInterval {
    #[allow(dead_code)]
    pub amount: u32,
    #[allow(dead_code)]
    pub interval_type: String,
}

#[derive(Debug, Clone)]
pub struct QuickAddParser {
    label_regex: Regex,
    priority_regex: Regex,
    assignee_regex: Regex,
    project_regex: Regex,
    repeat_regex: Regex,
    due_regex: Regex,
    start_regex: Regex,
    time_regex: Regex,
    date_keywords: AhoCorasick,
    weekday_keywords: AhoCorasick,
}

impl QuickAddParser {
    pub fn new() -> Self {
        let date_patterns = vec![
            "today",
            "tomorrow",
            "yesterday",
            "next week",
            "this week",
            "last week",
            "next month",
            "this month",
            "last month",
            "next year",
            "this year",
            "last year",
            "this weekend",
            "next weekend",
            "later this week",
            "later next week",
            "end of month",
            "end of week",
            "end of year",
        ];

        let weekdays = vec![
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
            "mon",
            "tue",
            "wed",
            "thu",
            "fri",
            "sat",
            "sun",
        ];

        Self {
            label_regex: Regex::new(r#"\*(?:"([^"]+)"|'([^']+)'|\[([^\]]+)\]|(\S+))"#).unwrap(),
            priority_regex: Regex::new(r"!([1-4]|[nlmhu])(?:\b|$)").unwrap(),
            assignee_regex: Regex::new(r#"@(?:"([^"]+)"|'([^']+)'|\[([^\]]+)\]|(\S+))"#).unwrap(),
            project_regex: Regex::new(r#"\+(?:"([^"]+)"|'([^']+)'|\[([^\]]+)\]|(\S+))"#).unwrap(),
            repeat_regex: Regex::new(r"every\s+(?:(\d+)\s+)?(\w+)").unwrap(),
            due_regex: Regex::new(r"(?i)\bdue\s+([^@+*!]+)").unwrap(),
            start_regex: Regex::new(r"(?i)\bstart(?::\s*|\s+)").unwrap(),
            time_regex: Regex::new(r"(?i)\bat\s+(\d{1,2})(?::(\d{2}))?\s*(am|pm)?\b").unwrap(),
            date_keywords: AhoCorasick::new(date_patterns).unwrap(),
            weekday_keywords: AhoCorasick::new(weekdays).unwrap(),
        }
    }

    fn resolve_priority(raw: &str) -> Option<String> {
        match raw {
            "n" => Some("none".to_string()),
            "l" | "1" => Some("low".to_string()),
            "m" | "2" => Some("medium".to_string()),
            "h" | "3" => Some("high".to_string()),
            "u" | "4" => Some("urgent".to_string()),
            _ => None,
        }
    }

    pub fn parse(&self, text: &str) -> ParsedTask {
        let mut task = ParsedTask {
            title: text.to_string(),
            labels: Vec::new(),
            assignees: Vec::new(),
            project: None,
            priority: None,
            due_date: None,
            start_date: None,
            repeat_interval: None,
        };

        for cap in self.label_regex.captures_iter(text) {
            let label = cap
                .get(1)
                .or(cap.get(2))
                .or(cap.get(3))
                .or(cap.get(4))
                .unwrap()
                .as_str();
            task.labels.push(label.to_string());
        }

        if let Some(cap) = self.priority_regex.captures(text) {
            task.priority = Self::resolve_priority(&cap[1]);
        }

        for cap in self.assignee_regex.captures_iter(text) {
            let assignee = cap
                .get(1)
                .or(cap.get(2))
                .or(cap.get(3))
                .or(cap.get(4))
                .unwrap()
                .as_str();
            task.assignees.push(assignee.to_string());
        }

        if let Some(cap) = self.project_regex.captures(text) {
            task.project = Some(
                cap.get(1)
                    .or(cap.get(2))
                    .or(cap.get(3))
                    .or(cap.get(4))
                    .unwrap()
                    .as_str()
                    .to_string(),
            );
        }

        if let Some(cap) = self.repeat_regex.captures(text) {
            let amount = cap
                .get(1)
                .map(|m| m.as_str().parse().unwrap_or(1))
                .unwrap_or(1);
            let interval_type = cap[2].to_string();
            task.repeat_interval = Some(RepeatInterval {
                amount,
                interval_type,
            });
        }

        let mut parsed_start_span = None;
        if let Some((start_text, start_span)) = self.extract_last_start_value(text) {
            let start_text_lower = start_text.to_lowercase();
            if start_text_lower == "eow" || start_text_lower == "end of week" {
                let now = Local::now();
                let current_weekday = now.weekday().num_days_from_sunday();
                let days_until_sunday = if current_weekday == 0 {
                    0
                } else {
                    7 - current_weekday
                };
                let sunday = now + Duration::days(days_until_sunday as i64);
                let naive = sunday.date_naive();
                task.start_date = naive
                    .and_hms_opt(23, 59, 59)
                    .map(|dt| dt.and_utc())
                    .or_else(|| naive.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc()))
                    .or_else(|| naive.and_hms_opt(12, 0, 0).map(|dt| dt.and_utc()));
                if task.start_date.is_none() {
                    let today = Local::now().date_naive();
                    task.start_date = today.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc());
                }
                parsed_start_span = Some(start_span);
            } else if start_text_lower == "eom" || start_text_lower == "end of month" {
                let now = Local::now();
                let mut last_day = now.date_naive();
                last_day = last_day.with_day(1).unwrap();
                last_day = last_day + Duration::days(32);
                last_day = last_day.with_day(1).unwrap();
                last_day = last_day - Duration::days(1);
                task.start_date = last_day
                    .and_hms_opt(23, 59, 59)
                    .map(|dt| dt.and_utc())
                    .or_else(|| last_day.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc()))
                    .or_else(|| last_day.and_hms_opt(12, 0, 0).map(|dt| dt.and_utc()));
                if task.start_date.is_none() {
                    let today = Local::now().date_naive();
                    task.start_date = today.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc());
                }
                parsed_start_span = Some(start_span);
            } else {
                task.start_date = self.parse_date(start_text);
                if task.start_date.is_some() {
                    parsed_start_span = Some(start_span);
                }
            }
        }

        if let Some(cap) = self.due_regex.captures(text) {
            task.due_date = self.parse_date(cap.get(1).unwrap().as_str());
        } else {
            task.due_date = self.parse_date(text);
        }

        let cleaned_title = self.clean_title(text, parsed_start_span);
        debug_log(&format!(
            "[MAGIC PARSER] Cleaned title: '{}', from input: '{}'",
            cleaned_title, text
        ));
        task.title = cleaned_title;

        task
    }

    fn parse_date(&self, text: &str) -> Option<DateTime<Utc>> {
        let text_lower = text.to_lowercase();
        let now = Local::now();

        if let Ok(parsed_date) = parse_date_string(&text, now.into(), Dialect::Us) {
            return Some(parsed_date);
        }

        if self.date_keywords.find(&text_lower).is_some() {
            return self.parse_date_keywords(&text_lower, now);
        }

        if self.weekday_keywords.find(&text_lower).is_some() {
            return self.parse_weekday(&text_lower, now);
        }

        if let Some(duration_date) = self.parse_duration_date(&text_lower, now) {
            return Some(duration_date);
        }

        if let Some(ordinal_date) = self.parse_ordinal_date(&text_lower, now) {
            return Some(ordinal_date);
        }

        self.parse_specific_date(text)
    }

    fn parse_date_keywords(
        &self,
        text: &str,
        now: chrono::DateTime<Local>,
    ) -> Option<DateTime<Utc>> {
        let target_time = self.extract_time(text).unwrap_or((23, 59));

        if text.contains("today") {
            Some(
                now.date_naive()
                    .and_hms_opt(target_time.0, target_time.1, 59)?
                    .and_utc(),
            )
        } else if text.contains("tomorrow") {
            Some(
                (now + Duration::days(1))
                    .date_naive()
                    .and_hms_opt(target_time.0, target_time.1, 59)?
                    .and_utc(),
            )
        } else if text.contains("yesterday") {
            Some(
                (now - Duration::days(1))
                    .date_naive()
                    .and_hms_opt(target_time.0, target_time.1, 59)?
                    .and_utc(),
            )
        } else if text.contains("this weekend") {
            let days_until_saturday = (6 - now.weekday().num_days_from_monday()) % 7;
            let saturday = now + Duration::days(days_until_saturday as i64);
            Some(
                saturday
                    .date_naive()
                    .and_hms_opt(target_time.0, target_time.1, 59)?
                    .and_utc(),
            )
        } else if text.contains("next weekend") {
            let days_until_next_saturday = 7 + (6 - now.weekday().num_days_from_monday()) % 7;
            let next_saturday = now + Duration::days(days_until_next_saturday as i64);
            Some(
                next_saturday
                    .date_naive()
                    .and_hms_opt(target_time.0, target_time.1, 59)?
                    .and_utc(),
            )
        } else if text.contains("next week") {
            Some(
                (now + Duration::weeks(1))
                    .date_naive()
                    .and_hms_opt(target_time.0, target_time.1, 59)?
                    .and_utc(),
            )
        } else if text.contains("this week") {
            let days_until_sunday = (7 - now.weekday().num_days_from_monday()) % 7;
            let sunday = now + Duration::days(days_until_sunday as i64);
            Some(
                sunday
                    .date_naive()
                    .and_hms_opt(target_time.0, target_time.1, 59)?
                    .and_utc(),
            )
        } else if text.contains("next month") {
            Some(
                (now + Duration::days(30))
                    .date_naive()
                    .and_hms_opt(target_time.0, target_time.1, 59)?
                    .and_utc(),
            )
        } else if text.contains("end of month") {
            let mut last_day = now.date_naive();
            last_day = last_day.with_day(1).unwrap();
            last_day = last_day + Duration::days(32);
            last_day = last_day.with_day(1).unwrap();
            last_day = last_day - Duration::days(1);
            Some(
                last_day
                    .and_hms_opt(target_time.0, target_time.1, 59)?
                    .and_utc(),
            )
        } else {
            None
        }
    }

    fn parse_weekday(&self, text: &str, now: chrono::DateTime<Local>) -> Option<DateTime<Utc>> {
        let target_time = self.extract_time(text).unwrap_or((23, 59));

        let weekdays = [
            ("monday", 0),
            ("mon", 0),
            ("tuesday", 1),
            ("tue", 1),
            ("wednesday", 2),
            ("wed", 2),
            ("thursday", 3),
            ("thu", 3),
            ("friday", 4),
            ("fri", 4),
            ("saturday", 5),
            ("sat", 5),
            ("sunday", 6),
            ("sun", 6),
        ];

        for (day_name, target_weekday) in &weekdays {
            if text.contains(day_name) {
                let current_weekday = now.weekday().num_days_from_monday();
                let days_ahead = if *target_weekday >= current_weekday {
                    *target_weekday - current_weekday
                } else {
                    7 - current_weekday + *target_weekday
                };

                let target_date = now + Duration::days(days_ahead as i64);
                return Some(
                    target_date
                        .date_naive()
                        .and_hms_opt(target_time.0, target_time.1, 59)?
                        .and_utc(),
                );
            }
        }

        None
    }

    fn parse_duration_date(
        &self,
        text: &str,
        now: chrono::DateTime<Local>,
    ) -> Option<DateTime<Utc>> {
        let duration_regex = Regex::new(r"in\s+(\d+)\s+(day|week|month|hour)s?").unwrap();

        if let Some(cap) = duration_regex.captures(text) {
            let amount: i64 = cap[1].parse().ok()?;
            let unit = &cap[2];
            let target_time = self.extract_time(text).unwrap_or((23, 59));

            let target_date = match unit {
                "hour" => now + Duration::hours(amount),
                "day" => now + Duration::days(amount),
                "week" => now + Duration::weeks(amount),
                "month" => now + Duration::days(amount * 30),
                _ => return None,
            };

            if unit == "hour" {
                Some(target_date.with_timezone(&Utc))
            } else {
                Some(
                    target_date
                        .date_naive()
                        .and_hms_opt(target_time.0, target_time.1, 59)?
                        .and_utc(),
                )
            }
        } else {
            None
        }
    }

    fn parse_ordinal_date(
        &self,
        text: &str,
        now: chrono::DateTime<Local>,
    ) -> Option<DateTime<Utc>> {
        let ordinal_regex = Regex::new(r"(\d{1,2})(?:st|nd|rd|th)").unwrap();

        if let Some(cap) = ordinal_regex.captures(text) {
            let day: u32 = cap[1].parse().ok()?;
            let target_time = self.extract_time(text).unwrap_or((23, 59));

            let target_date = now.date_naive().with_day(day)?;
            Some(
                target_date
                    .and_hms_opt(target_time.0, target_time.1, 59)?
                    .and_utc(),
            )
        } else {
            None
        }
    }

    fn extract_time(&self, text: &str) -> Option<(u32, u32)> {
        if let Some(cap) = self.time_regex.captures(text) {
            let hour: u32 = cap[1].parse().ok()?;
            let minute: u32 = cap
                .get(2)
                .map(|m| m.as_str().parse().unwrap_or(0))
                .unwrap_or(0);
            let am_pm = cap.get(3).map(|m| m.as_str().to_lowercase());

            let adjusted_hour = match am_pm.as_deref() {
                Some("pm") if hour != 12 => hour + 12,
                Some("am") if hour == 12 => 0,
                _ => hour,
            };

            if adjusted_hour < 24 && minute < 60 {
                Some((adjusted_hour, minute))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn parse_specific_date(&self, text: &str) -> Option<DateTime<Utc>> {
        let target_time = self.extract_time(text).unwrap_or((23, 59));

        if let Some(caps) = Regex::new(r"(\d{1,2})/(\d{1,2})/(\d{4})")
            .unwrap()
            .captures(text)
        {
            let day: u32 = caps[1].parse().ok()?;
            let month: u32 = caps[2].parse().ok()?;
            let year: i32 = caps[3].parse().ok()?;
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                return Some(
                    date.and_hms_opt(target_time.0, target_time.1, 59)?
                        .and_utc(),
                );
            }
        }

        if let Some(caps) = Regex::new(r"(\d{4})-(\d{1,2})-(\d{1,2})")
            .unwrap()
            .captures(text)
        {
            let year: i32 = caps[1].parse().ok()?;
            let month: u32 = caps[2].parse().ok()?;
            let day: u32 = caps[3].parse().ok()?;
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                return Some(
                    date.and_hms_opt(target_time.0, target_time.1, 59)?
                        .and_utc(),
                );
            }
        }

        self.parse_month_name_date(text)
    }

    fn parse_month_name_date(&self, text: &str) -> Option<DateTime<Utc>> {
        let month_regex = Regex::new(
            r"(?i)(jan|january|feb|february|mar|march|apr|april|may|jun|june|jul|july|aug|august|sep|september|oct|october|nov|november|dec|december)\s+(\d{1,2})(?:st|nd|rd|th)?"
        ).unwrap();

        if let Some(caps) = month_regex.captures(text) {
            let month_str = caps[1].to_lowercase();
            let day: u32 = caps[2].parse().ok()?;
            let target_time = self.extract_time(text).unwrap_or((23, 59));

            let month_num = match month_str.as_str() {
                "jan" | "january" => 1,
                "feb" | "february" => 2,
                "mar" | "march" => 3,
                "apr" | "april" => 4,
                "may" => 5,
                "jun" | "june" => 6,
                "jul" | "july" => 7,
                "aug" | "august" => 8,
                "sep" | "september" => 9,
                "oct" | "october" => 10,
                "nov" | "november" => 11,
                "dec" | "december" => 12,
                _ => return None,
            };

            let current_year = Local::now().year();
            let date = NaiveDate::from_ymd_opt(current_year, month_num, day)?;
            Some(
                date.and_hms_opt(target_time.0, target_time.1, 59)?
                    .and_utc(),
            )
        } else {
            None
        }
    }

    fn clean_title(&self, text: &str, start_span: Option<(usize, usize)>) -> String {
        let mut cleaned = text.to_string();
        if let Some((start, end)) = start_span {
            cleaned.replace_range(start..end, "");
        }
        cleaned = self.label_regex.replace_all(&cleaned, "").to_string();
        cleaned = self.priority_regex.replace_all(&cleaned, "").to_string();
        cleaned = self.assignee_regex.replace_all(&cleaned, "").to_string();
        cleaned = self.project_regex.replace_all(&cleaned, "").to_string();
        cleaned = self.repeat_regex.replace_all(&cleaned, "").to_string();

        cleaned = self.due_regex.replace_all(&cleaned, "").to_string();

        cleaned = self.remove_date_text(&cleaned);

        cleaned = self.time_regex.replace_all(&cleaned, "").to_string();

        cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn remove_date_text(&self, text: &str) -> String {
        let mut cleaned = text.to_string();

        let date_patterns = [
            r"(?i)\blater\s+(this|next)\s+week\b",
            r"(?i)\bend\s+of\s+(week|month|year)\b",
            r"(?i)\bin\s+\d+\s+(day|week|month|hour)s?\b",
            r"(?i)\bnext\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b",
            r"(?i)\b(this|next|last)\s+(week|month|year|weekend)\b",
            r"(?i)\b(today|tomorrow|yesterday)\b",
            r"(?i)\b(monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b",
            r"(?i)\b(mon|tue|wed|thu|fri|sat|sun)\b",
            r"(?i)\b(jan|january|feb|february|mar|march|apr|april|may|jun|june|jul|july|aug|august|sep|september|oct|october|nov|november|dec|december)\s+\d{1,2}(?:st|nd|rd|th)?\b",
            r"\b\d{1,2}/\d{1,2}/\d{4}\b",
            r"\b\d{4}-\d{1,2}-\d{1,2}\b",
            r"\b\d{1,2}(?:st|nd|rd|th)\b",
        ];

        for pattern in &date_patterns {
            let regex = Regex::new(pattern).unwrap();
            cleaned = regex.replace_all(&cleaned, "").to_string();
        }

        cleaned
    }

    fn extract_last_start_value<'a>(&self, text: &'a str) -> Option<(&'a str, (usize, usize))> {
        let mut last = None;

        for marker in self.start_regex.find_iter(text) {
            let value_start = marker.end();
            let (value, value_end) = self.extract_magic_value(text, value_start);
            if !value.is_empty() {
                last = Some((value, (marker.start(), value_end)));
            }
        }

        last
    }

    fn extract_magic_value<'a>(&self, text: &'a str, start: usize) -> (&'a str, usize) {
        let remainder = &text[start..];
        let trimmed = remainder.trim_start();
        let leading_ws = remainder.len() - trimmed.len();
        let value_start = start + leading_ws;

        if trimmed.is_empty() {
            return ("", start);
        }

        let mut chars = trimmed.char_indices();
        if let Some((_, first)) = chars.next() {
            match first {
                '"' => {
                    if let Some((idx, _)) = trimmed[1..].char_indices().find(|(_, c)| *c == '"') {
                        let end = value_start + idx + 2;
                        return (&text[value_start + 1..end - 1], end);
                    }
                    return (trimmed[1..].trim(), text.len());
                }
                '\'' => {
                    if let Some((idx, _)) = trimmed[1..].char_indices().find(|(_, c)| *c == '\'') {
                        let end = value_start + idx + 2;
                        return (&text[value_start + 1..end - 1], end);
                    }
                    return (trimmed[1..].trim(), text.len());
                }
                '[' => {
                    if let Some((idx, _)) = trimmed[1..].char_indices().find(|(_, c)| *c == ']') {
                        let end = value_start + idx + 2;
                        return (&text[value_start + 1..end - 1], end);
                    }
                    return (trimmed[1..].trim(), text.len());
                }
                _ => {}
            }
        }

        let mut end = text.len();
        for (idx, ch) in trimmed.char_indices() {
            if ch.is_whitespace() {
                let next = trimmed[idx..].trim_start();
                if next.starts_with('@')
                    || next.starts_with('+')
                    || next.starts_with('*')
                    || next.starts_with('!')
                    || next.to_ascii_lowercase().starts_with("every ")
                    || next.eq_ignore_ascii_case("every")
                    || next.to_ascii_lowercase().starts_with("due ")
                    || next.eq_ignore_ascii_case("due")
                    || next.to_ascii_lowercase().starts_with("start:")
                    || next.to_ascii_lowercase().starts_with("start ")
                {
                    end = value_start + idx;
                    break;
                }
            }
        }

        (text[value_start..end].trim_end(), end)
    }
}

#[cfg(test)]
#[test]
fn test_start_eow_and_tomorrow() {
    let parser = QuickAddParser::new();
    let task_eow = parser.parse("Start project start:eow");
    assert!(task_eow.start_date.is_some());
    let task_eom = parser.parse("Start project start:eom");
    assert!(task_eom.start_date.is_some());
    let task_tomorrow = parser.parse("Start project start:tomorrow");
    assert!(task_tomorrow.start_date.is_some());
}

#[cfg(test)]
mod tests {
    use super::QuickAddParser;

    #[test]
    fn test_parse_task_with_magic() {
        let parser = QuickAddParser::new();
        let task = parser.parse("Buy groceries *shopping @john +personal tomorrow !2");

        assert_eq!(task.title, "Buy groceries");
        assert_eq!(task.labels, vec!["shopping"]);
        assert_eq!(task.assignees, vec!["john"]);
        assert_eq!(task.project, Some("personal".to_string()));
        assert_eq!(task.priority, Some("medium".to_string()));
        assert!(task.due_date.is_some());
    }

    #[test]
    fn test_parse_priority_letters() {
        let parser = QuickAddParser::new();

        let task_n = parser.parse("Task !n");
        assert_eq!(task_n.priority, Some("none".to_string()));

        let task_l = parser.parse("Task !l");
        assert_eq!(task_l.priority, Some("low".to_string()));

        let task_m = parser.parse("Task !m");
        assert_eq!(task_m.priority, Some("medium".to_string()));

        let task_h = parser.parse("Task !h");
        assert_eq!(task_h.priority, Some("high".to_string()));

        let task_u = parser.parse("Task !u");
        assert_eq!(task_u.priority, Some("urgent".to_string()));
    }

    #[test]
    fn test_parse_priority_numbers_compat() {
        let parser = QuickAddParser::new();

        let task_1 = parser.parse("Task !1");
        assert_eq!(task_1.priority, Some("low".to_string()));

        let task_2 = parser.parse("Task !2");
        assert_eq!(task_2.priority, Some("medium".to_string()));

        let task_3 = parser.parse("Task !3");
        assert_eq!(task_3.priority, Some("high".to_string()));

        let task_4 = parser.parse("Task !4");
        assert_eq!(task_4.priority, Some("urgent".to_string()));
    }

    #[test]
    fn test_parse_labels_with_spaces() {
        let parser = QuickAddParser::new();
        let task = parser.parse(r#"Task with *"label with spaces" and *simple"#);

        assert_eq!(task.labels, vec!["label with spaces", "simple"]);
    }

    #[test]
    fn test_parse_repeat_interval() {
        let parser = QuickAddParser::new();
        let task = parser.parse("Daily standup every 2 days");

        assert!(task.repeat_interval.is_some());
        let repeat = task.repeat_interval.unwrap();
        assert_eq!(repeat.amount, 2);
        assert_eq!(repeat.interval_type, "days");
    }

    #[test]
    fn test_enhanced_date_parsing() {
        let parser = QuickAddParser::new();

        let task1 = parser.parse("Meeting tomorrow at 2:30pm");
        assert!(task1.due_date.is_some());
        assert_eq!(task1.title, "Meeting");

        let task2 = parser.parse("Call mom next friday");
        assert!(task2.due_date.is_some());
        assert_eq!(task2.title, "Call mom");

        let task3 = parser.parse("Pay rent 15th");
        assert!(task3.due_date.is_some());
        assert_eq!(task3.title, "Pay rent");

        let task4 = parser.parse("Follow up in 3 days");
        assert!(task4.due_date.is_some());
        assert_eq!(task4.title, "Follow up");
    }

    #[test]
    fn test_complex_parsing() {
        let parser = QuickAddParser::new();
        let task = parser.parse(
            r#"Review proposal *urgent *"high priority" @jane @"john doe" +"Client Work" next monday at 10am !4 every week"#,
        );

        assert_eq!(task.title, "Review proposal");
        assert_eq!(task.labels, vec!["urgent", "high priority"]);
        assert_eq!(task.assignees, vec!["jane", "john doe"]);
        assert_eq!(task.project, Some("Client Work".to_string()));
        assert_eq!(task.priority, Some("urgent".to_string()));
        assert!(task.due_date.is_some());
        assert!(task.repeat_interval.is_some());
    }

    #[test]
    fn test_month_name_parsing() {
        let parser = QuickAddParser::new();
        let task = parser.parse("Submit report Feb 17th at 5pm");

        assert_eq!(task.title, "Submit report");
        assert!(task.due_date.is_some());
    }

    #[test]
    fn test_weekend_parsing() {
        let parser = QuickAddParser::new();
        let task = parser.parse("Clean garage this weekend");

        assert_eq!(task.title, "Clean garage");
        assert!(task.due_date.is_some());
    }

    #[test]
    fn test_time_only_parsing() {
        let parser = QuickAddParser::new();
        let task = parser.parse("Team meeting at 10:30am *important");

        assert_eq!(task.title, "Team meeting");
        assert_eq!(task.labels, vec!["important"]);
    }

    #[test]
    fn test_start_requires_explicit_marker() {
        let parser = QuickAddParser::new();
        let task = parser.parse("Start project +Internes @Charles");

        assert_eq!(task.title, "Start project");
        assert_eq!(task.project, Some("Internes".to_string()));
        assert_eq!(task.assignees, vec!["Charles"]);
        assert!(task.start_date.is_none());
    }

    #[test]
    fn test_start_supports_multi_word_dates() {
        let parser = QuickAddParser::new();
        let task = parser.parse("Review rollout +Internes start:end of week @Charles");

        assert_eq!(task.title, "Review rollout");
        assert_eq!(task.project, Some("Internes".to_string()));
        assert_eq!(task.assignees, vec!["Charles"]);
        assert!(task.start_date.is_some());
    }
}
