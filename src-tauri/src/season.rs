use lazy_static::lazy_static;
use regex::Regex;
lazy_static! {
    // Match the field itself, not the first unrelated [timestamp] in a line.
    static ref SEASON: Regex = Regex::new(r"(?:\+player\+SeasonId|\+SeasonId)\s*\[([0-9]{1,6})\]").unwrap();
}
pub fn from_log_line(line: &str) -> Option<u32> {
    SEASON
        .captures(line)?
        .get(1)?
        .as_str()
        .parse::<u32>()
        .ok()
        .filter(|n| *n > 0 && *n <= 100_000)
}
#[cfg(test)]
mod tests {
    #[test]
    fn field_not_timestamp_and_future_season_is_not_aliased() {
        assert_eq!(
            super::from_log_line("[2026] | +player+SeasonId [1601]"),
            Some(1601)
        );
        assert_eq!(super::from_log_line("|      +SeasonId [1701]"), Some(1701));
        assert_eq!(super::from_log_line("unrelated SeasonId [1601]"), None);
    }
}
