//! Display helpers for the frontend.

/// Turns an RFC 3339 timestamp into a compact UTC date/time string.
///
/// The server always sends UTC (`2026-11-19T12:52:47.123456+00:00`); showing
/// the date and time down to seconds is enough for the UI. Values shorter than
/// 19 bytes are returned unchanged.
pub fn format_time(value: &str) -> String {
    value
        .get(..19)
        .map(|prefix| prefix.replace('T', " "))
        .unwrap_or_else(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_timestamps() {
        assert_eq!(
            format_time("2026-11-19T12:52:47.123456+00:00"),
            "2026-11-19 12:52:47"
        );
        assert_eq!(format_time("not-a-date"), "not-a-date");
    }
}
