//! Normalizes messy byte-size and duration strings into exact values,
//! and formats values back into compact, consistent strings.

mod bytes;
mod duration;

pub use bytes::{format_bytes, parse_bytes, ParseByteSizeError};
pub use duration::{format_duration, parse_duration, ParseDurationError};
