//! Parsing and formatting for byte sizes.

use std::fmt;

const KB: f64 = 1000.0;
const MB: f64 = KB * 1000.0;
const GB: f64 = MB * 1000.0;
const TB: f64 = GB * 1000.0;

const KIB: f64 = 1024.0;
const MIB: f64 = KIB * 1024.0;
const GIB: f64 = MIB * 1024.0;
const TIB: f64 = GIB * 1024.0;

// Bit units are counted in bits, then divided by 8 to land in bytes like
// everything else this module produces. Values are powers of 1000, matching
// how bandwidth is normally advertised (a "1 Gbit" link is 1e9 bits/s, not
// 2^30).
const KBIT: f64 = 1000.0;
const MBIT: f64 = KBIT * 1000.0;
const GBIT: f64 = MBIT * 1000.0;
const TBIT: f64 = GBIT * 1000.0;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseByteSizeError(String);

impl fmt::Display for ParseByteSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid byte size: {}", self.0)
    }
}

impl std::error::Error for ParseByteSizeError {}

/// Parses a human-written byte size into an exact byte count.
///
/// Accepts decimal units (`kb`, `mb`, `gb`, `tb`) at powers of 1000 and
/// binary units (`kib`, `mib`, `gib`, `tib`) at powers of 1024. Unit
/// matching is case-insensitive, surrounding and inner whitespace is
/// ignored, underscores between digits are dropped, and a comma is
/// accepted as a decimal separator ("1,5 mb"). A bare number with no
/// unit is read as a count of bytes.
///
/// Also accepts bit-based units for network throughput figures (`bit`,
/// `kbit`, `mbit`, `gbit`, `tbit`, and the `bps`/`kbps`/`mbps`/`gbps`/`tbps`
/// spellings), at powers of 1000, converted to bytes by dividing by 8.
/// Since unit matching is case-insensitive there's no way to tell "10 Mb"
/// meant megabits rather than megabytes, so the short form always means
/// bytes - spell out `mbit` or `mbps` for bits.
pub fn parse_bytes(input: &str) -> Result<u64, ParseByteSizeError> {
    let original = input;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ParseByteSizeError(original.to_string()));
    }

    let mut chars = trimmed.chars().peekable();

    if let Some(&c) = chars.peek() {
        if c == '-' {
            return Err(ParseByteSizeError(original.to_string()));
        }
        if c == '+' {
            chars.next();
        }
    }

    let mut number = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || c == '.' || c == ',' || c == '_' {
            if c != '_' {
                number.push(if c == ',' { '.' } else { c });
            }
            chars.next();
        } else {
            break;
        }
    }

    if number.is_empty() {
        return Err(ParseByteSizeError(original.to_string()));
    }

    let value: f64 = number
        .parse()
        .map_err(|_| ParseByteSizeError(original.to_string()))?;

    let rest: String = chars.collect();
    let unit = rest.trim().to_ascii_lowercase();

    let multiplier = match unit.as_str() {
        "" | "b" | "byte" | "bytes" => 1.0,
        "k" | "kb" => KB,
        "m" | "mb" => MB,
        "g" | "gb" => GB,
        "t" | "tb" => TB,
        "ki" | "kib" => KIB,
        "mi" | "mib" => MIB,
        "gi" | "gib" => GIB,
        "ti" | "tib" => TIB,
        "bit" | "bits" | "bps" => 1.0 / 8.0,
        "kbit" | "kbits" | "kbps" => KBIT / 8.0,
        "mbit" | "mbits" | "mbps" => MBIT / 8.0,
        "gbit" | "gbits" | "gbps" => GBIT / 8.0,
        "tbit" | "tbits" | "tbps" => TBIT / 8.0,
        _ => return Err(ParseByteSizeError(original.to_string())),
    };

    let bytes = value * multiplier;
    if !bytes.is_finite() || bytes < 0.0 {
        return Err(ParseByteSizeError(original.to_string()));
    }

    Ok(bytes.round() as u64)
}

/// Formats a byte count using binary units (KiB, MiB, GiB, TiB), picking
/// the largest unit that keeps the value at 1.0 or above.
pub fn format_bytes(bytes: u64) -> String {
    let value = bytes as f64;
    let (scaled, suffix) = if value >= TIB {
        (value / TIB, "TiB")
    } else if value >= GIB {
        (value / GIB, "GiB")
    } else if value >= MIB {
        (value / MIB, "MiB")
    } else if value >= KIB {
        (value / KIB, "KiB")
    } else {
        (value, "B")
    };

    if suffix == "B" {
        format!("{} B", bytes)
    } else {
        format!("{:.2} {}", scaled, suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bytes_table() {
        let cases: &[(&str, u64)] = &[
            ("0", 0),
            ("1024", 1024),
            ("  1024  ", 1024),
            ("+1024", 1024),
            ("500 bytes", 500),
            ("1 byte", 1),
            ("1b", 1),
            ("10MB", 10_000_000),
            ("10mb", 10_000_000),
            ("10Mb", 10_000_000),
            ("10 MB", 10_000_000),
            ("10  MB", 10_000_000),
            ("1.5MB", 1_500_000),
            ("1,5MB", 1_500_000),
            ("1_000 bytes", 1_000),
            ("1GiB", 1_073_741_824),
            ("1.5GiB", 1_610_612_736),
            ("2Ki", 2_048),
            ("2K", 2_000),
            ("2k", 2_000),
            ("8bit", 1),
            ("8000 bits", 1_000),
            ("1kbit", 125),
            ("1Mbit", 125_000),
            ("100mbps", 12_500_000),
            ("1Gbps", 125_000_000),
        ];

        for (input, expected) in cases {
            let got = parse_bytes(input).unwrap_or_else(|e| panic!("{input:?}: {e}"));
            assert_eq!(got, *expected, "input: {input:?}");
        }
    }

    #[test]
    fn parse_bytes_rejects_garbage() {
        let cases = ["", "   ", "-5MB", "MB", "5XB", "five MB", "5..5MB"];
        for input in cases {
            assert!(parse_bytes(input).is_err(), "expected error for {input:?}");
        }
    }

    #[test]
    fn format_bytes_table() {
        let cases: &[(u64, &str)] = &[
            (0, "0 B"),
            (512, "512 B"),
            (1_024, "1.00 KiB"),
            (1_536, "1.50 KiB"),
            (1_048_576, "1.00 MiB"),
            (1_073_741_824, "1.00 GiB"),
        ];

        for (input, expected) in cases {
            assert_eq!(format_bytes(*input), *expected, "input: {input}");
        }
    }
}
