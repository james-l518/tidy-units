//! Parsing and formatting for durations.

use std::fmt;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseDurationError(String);

impl fmt::Display for ParseDurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid duration: {}", self.0)
    }
}

impl std::error::Error for ParseDurationError {}

/// Parses a human-written duration into a `Duration`.
///
/// Accepts a compact unit form ("1h30m", "90s", "500ms", "1.5h"), a
/// colon-separated clock form ("1:30:00", "5:09"), and a bare number
/// with no unit read as seconds. Unit matching is case-insensitive,
/// whitespace between tokens is ignored, and a comma is accepted as a
/// decimal separator. A trailing number with no unit is read as
/// seconds, so "1h30" means one hour and thirty seconds, not thirty
/// minutes - always write the unit for anything but the last token.
pub fn parse_duration(input: &str) -> Result<Duration, ParseDurationError> {
    let original = input;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ParseDurationError(original.to_string()));
    }
    if trimmed.starts_with('-') {
        return Err(ParseDurationError(original.to_string()));
    }

    let total_seconds = if trimmed.contains(':') {
        parse_clock(trimmed).ok_or_else(|| ParseDurationError(original.to_string()))?
    } else {
        parse_units(trimmed).ok_or_else(|| ParseDurationError(original.to_string()))?
    };

    if !total_seconds.is_finite() || total_seconds < 0.0 {
        return Err(ParseDurationError(original.to_string()));
    }

    Ok(Duration::from_secs_f64(total_seconds))
}

fn parse_clock(input: &str) -> Option<f64> {
    let parts: Vec<&str> = input.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }

    let mut values = Vec::with_capacity(parts.len());
    for part in &parts {
        let cleaned = part.trim().replace(',', ".");
        if cleaned.is_empty() {
            return None;
        }
        values.push(cleaned.parse::<f64>().ok()?);
    }

    let total = if values.len() == 3 {
        values[0] * 3600.0 + values[1] * 60.0 + values[2]
    } else {
        values[0] * 60.0 + values[1]
    };

    Some(total)
}

fn parse_units(input: &str) -> Option<f64> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut total = 0.0;
    let mut saw_token = false;

    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }

        let mut number = String::new();
        while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == ',') {
            number.push(if chars[i] == ',' { '.' } else { chars[i] });
            i += 1;
        }
        if number.is_empty() {
            return None;
        }
        let value: f64 = number.parse().ok()?;

        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        let unit_start = i;
        while i < chars.len() && chars[i].is_alphabetic() {
            i += 1;
        }
        let unit: String = chars[unit_start..i]
            .iter()
            .collect::<String>()
            .to_ascii_lowercase();

        let multiplier = match unit.as_str() {
            "" | "s" | "sec" | "secs" | "second" | "seconds" => 1.0,
            "ms" | "milli" | "millis" | "millisecond" | "milliseconds" => 0.001,
            "m" | "min" | "mins" | "minute" | "minutes" => 60.0,
            "h" | "hr" | "hrs" | "hour" | "hours" => 3600.0,
            "d" | "day" | "days" => 86_400.0,
            _ => return None,
        };

        total += value * multiplier;
        saw_token = true;
    }

    if saw_token {
        Some(total)
    } else {
        None
    }
}

/// Formats a duration compactly, dropping leading zero components
/// ("1h30m0s", "1m30s", "45s"). Durations under one second are shown
/// in milliseconds.
pub fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let nanos = d.subsec_nanos();

    if total_secs == 0 {
        if nanos == 0 {
            return "0s".to_string();
        }
        let millis = nanos as f64 / 1_000_000.0;
        return format!("{}ms", trim_trailing_zeros(&format!("{:.3}", millis)));
    }

    let days = total_secs / 86_400;
    let hours = (total_secs % 86_400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    let mut out = String::new();
    if days > 0 {
        out.push_str(&format!("{}d", days));
    }
    if hours > 0 || days > 0 {
        out.push_str(&format!("{}h", hours));
    }
    if minutes > 0 || hours > 0 || days > 0 {
        out.push_str(&format!("{}m", minutes));
    }

    if nanos > 0 {
        let frac_secs = seconds as f64 + nanos as f64 / 1_000_000_000.0;
        out.push_str(&format!(
            "{}s",
            trim_trailing_zeros(&format!("{:.3}", frac_secs))
        ));
    } else {
        out.push_str(&format!("{}s", seconds));
    }

    out
}

fn trim_trailing_zeros(s: &str) -> String {
    if !s.contains('.') {
        return s.to_string();
    }
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_duration_table() {
        let cases: &[(&str, Duration)] = &[
            ("90s", Duration::from_secs(90)),
            ("1h30m", Duration::from_secs(5_400)),
            ("90m", Duration::from_secs(5_400)),
            ("1.5h", Duration::from_secs(5_400)),
            ("1,5h", Duration::from_secs(5_400)),
            ("1:30:00", Duration::from_secs(5_400)),
            ("5:09", Duration::from_secs(309)),
            ("0:30", Duration::from_secs(30)),
            ("500ms", Duration::from_millis(500)),
            ("1d2h", Duration::from_secs(93_600)),
            ("1 h 30 m", Duration::from_secs(5_400)),
            ("1H30M", Duration::from_secs(5_400)),
            ("45", Duration::from_secs(45)),
            ("1h30", Duration::from_secs(3_630)),
        ];

        for (input, expected) in cases {
            let got = parse_duration(input).unwrap_or_else(|e| panic!("{input:?}: {e}"));
            assert_eq!(got, *expected, "input: {input:?}");
        }
    }

    #[test]
    fn parse_duration_rejects_garbage() {
        let cases = ["", "   ", "-1h", "10xyz", "1::30", "abc", "1:2:3:4"];
        for input in cases {
            assert!(
                parse_duration(input).is_err(),
                "expected error for {input:?}"
            );
        }
    }

    #[test]
    fn format_duration_table() {
        let cases: &[(Duration, &str)] = &[
            (Duration::from_secs(0), "0s"),
            (Duration::from_millis(500), "500ms"),
            (Duration::from_secs(45), "45s"),
            (Duration::from_secs(90), "1m30s"),
            (Duration::from_secs(5_400), "1h30m0s"),
            (Duration::from_secs(93_600), "1d2h0m0s"),
        ];

        for (input, expected) in cases {
            assert_eq!(format_duration(*input), *expected, "input: {input:?}");
        }
    }
}
