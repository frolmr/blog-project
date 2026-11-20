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
