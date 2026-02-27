#[derive(Debug, Copy, Clone, PartialEq)]
enum ComponentType {
    Year,
    Month,
    Day,
}

/// Represents the parsed structure of a date format
struct DateFormatInfo {
    /// Order of components (e.g., [Month, Day, Year] for "MM/DD/YYYY")
    components: [ComponentType; 3],
    /// Separator character (e.g., '/' or '-')
    separator: char,
}

impl DateFormatInfo {
    fn parse(format: &str) -> Option<Self> {
        // Find the separator (first non-alphabetic character)
        let separator = format.chars().find(|c| !c.is_ascii_alphabetic())?;

        // Split format by separator
        let parts: Vec<&str> = format.split(separator).collect();
        if parts.len() != 3 {
            return None;
        }

        let mut components = [ComponentType::Year; 3];
        for (i, part) in parts.iter().enumerate() {
            components[i] = if part.contains("YYYY") {
                ComponentType::Year
            } else if part.contains("MM") {
                ComponentType::Month
            } else if part.contains("DD") {
                ComponentType::Day
            } else {
                return None;
            };
        }

        Some(DateFormatInfo {
            components,
            separator,
        })
    }
}

fn is_valid_date_prefix(s: &str, format: &str) -> bool {
    let format_info = match DateFormatInfo::parse(format) {
        Some(info) => info,
        None => return false,
    };

    let bytes = s.as_bytes();
    let separator_byte = format_info.separator as u8;

    let mut component_index = 0;
    let mut value: u32 = 0;
    let mut digits = 0;

    // Track parsed values for day validation
    let mut year: Option<u32> = None;
    let mut month: Option<u32> = None;
    let mut day: Option<u32> = None;

    for &b in bytes {
        if component_index >= 3 {
            return false;
        }

        let current_component = format_info.components[component_index];

        match b {
            b'0'..=b'9' => {
                value = value * 10 + (b - b'0') as u32;
                digits += 1;

                match current_component {
                    ComponentType::Year => {
                        if digits > 4 {
                            return false;
                        }
                    }
                    ComponentType::Month => {
                        if digits > 2 || value > 12 {
                            return false;
                        }
                        // Require leading zero for single-digit months
                        if digits == 1 && value > 1 {
                            return false;
                        }
                        if digits == 2 && value == 0 {
                            return false;
                        }
                    }
                    ComponentType::Day => {
                        if digits > 2 {
                            return false;
                        }
                        // Require leading zero for single-digit days
                        if digits == 1 && value > 3 {
                            return false;
                        }
                        // Basic validation for day value (1-31 range)
                        if digits == 2 && (value == 0 || value > 31) {
                            return false;
                        }
                    }
                }
            }

            _ if b == separator_byte => {
                // Separator after the last component is not allowed
                if component_index == 2 {
                    return false;
                }

                // Validate component is complete before moving to next
                match current_component {
                    ComponentType::Year => {
                        if digits != 4 {
                            return false;
                        }
                        year = Some(value);
                    }
                    ComponentType::Month => {
                        if digits != 2 || value == 0 || value > 12 {
                            return false;
                        }
                        month = Some(value);
                    }
                    ComponentType::Day => {
                        if digits != 2 || value == 0 || value > 31 {
                            return false;
                        }
                        day = Some(value);
                    }
                }

                value = 0;
                digits = 0;
                component_index += 1;
            }

            _ => return false,
        }
    }

    // End-of-input validation
    if component_index >= 3 {
        return false;
    }

    // Store final component value if complete
    let current_component = format_info.components[component_index];
    match current_component {
        ComponentType::Year => {
            if digits == 4 {
                year = Some(value);
            }
        }
        ComponentType::Month => {
            if digits == 2 && (1..=12).contains(&value) {
                month = Some(value);
            }
        }
        ComponentType::Day => {
            if digits == 2 && (1..=31).contains(&value) {
                day = Some(value);
            }
        }
    }

    // If we have all three components, validate the day for the specific month/year
    if let (Some(y), Some(m), Some(d)) = (year, month, day) {
        if !valid_day(y, m, d) {
            return false;
        }
    }

    // Basic prefix validation
    match current_component {
        ComponentType::Year => digits <= 4,
        ComponentType::Month => digits <= 2 && value <= 12,
        ComponentType::Day => digits <= 2 && value <= 31,
    }
}

/// Append a character to a date string, auto-inserting separator if needed.
/// Returns Some(new_string) if the result is a valid date prefix, None otherwise.
///
/// Examples (format = "YYYY-MM-DD"):
/// - current="2025", c='0' -> Some("2025-0") (auto-inserts separator)
/// - current="2025-0", c='1' -> Some("2025-01")
/// - current="2025-01", c='1' -> Some("2025-01-1") (auto-inserts separator)
pub fn append_date_char(current: &str, c: char, format: &str) -> Option<String> {
    // First, try appending directly
    let direct = format!("{}{}", current, c);
    if is_valid_date_prefix(&direct, format) {
        return Some(direct);
    }

    // If direct append failed, try inserting separator first
    let format_info = DateFormatInfo::parse(format)?;
    let with_separator = format!("{}{}{}", current, format_info.separator, c);
    if is_valid_date_prefix(&with_separator, format) {
        return Some(with_separator);
    }

    None
}

fn valid_day(year: u32, month: u32, day: u32) -> bool {
    if day == 0 {
        return false;
    }

    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => day <= 31,
        4 | 6 | 9 | 11 => day <= 30,
        2 => {
            if is_leap_year(year) {
                day <= 29
            } else {
                day <= 28
            }
        }
        _ => false,
    }
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for YYYY-MM-DD format (ISO)
    #[test]
    fn valid_date_prefixes_iso() {
        let fmt = "YYYY-MM-DD";
        assert!(is_valid_date_prefix("2", fmt));
        assert!(is_valid_date_prefix("20", fmt));
        assert!(is_valid_date_prefix("202", fmt));
        assert!(is_valid_date_prefix("2026", fmt));
        assert!(is_valid_date_prefix("2026-", fmt));
        assert!(is_valid_date_prefix("2026-0", fmt));
        assert!(is_valid_date_prefix("2026-01", fmt));
        assert!(is_valid_date_prefix("2026-01-", fmt));
        assert!(is_valid_date_prefix("2026-01-0", fmt));
        assert!(is_valid_date_prefix("2026-01-05", fmt));
    }

    #[test]
    fn leap_year_handling_iso() {
        let fmt = "YYYY-MM-DD";
        assert!(is_valid_date_prefix("2024-02-29", fmt));
        assert!(!is_valid_date_prefix("2025-02-29", fmt));
    }

    #[test]
    fn rejects_single_digits_months_and_days_iso() {
        let fmt = "YYYY-MM-DD";
        assert!(!is_valid_date_prefix("2024-2", fmt));
        assert!(!is_valid_date_prefix("2025-02-4", fmt));
    }

    // Tests for MM/DD/YYYY format (US)
    #[test]
    fn valid_date_prefixes_us() {
        let fmt = "MM/DD/YYYY";
        assert!(is_valid_date_prefix("0", fmt));
        assert!(is_valid_date_prefix("01", fmt));
        assert!(is_valid_date_prefix("01/", fmt));
        assert!(is_valid_date_prefix("01/0", fmt));
        assert!(is_valid_date_prefix("01/15", fmt));
        assert!(is_valid_date_prefix("01/15/", fmt));
        assert!(is_valid_date_prefix("01/15/2", fmt));
        assert!(is_valid_date_prefix("01/15/20", fmt));
        assert!(is_valid_date_prefix("01/15/202", fmt));
        assert!(is_valid_date_prefix("01/15/2026", fmt));
    }

    #[test]
    fn leap_year_handling_us() {
        let fmt = "MM/DD/YYYY";
        assert!(is_valid_date_prefix("02/29/2024", fmt));
        assert!(!is_valid_date_prefix("02/29/2025", fmt));
    }

    #[test]
    fn rejects_single_digits_us() {
        let fmt = "MM/DD/YYYY";
        assert!(!is_valid_date_prefix("2", fmt)); // month starts with 2, but 2 > 1 so invalid
        assert!(!is_valid_date_prefix("01/5", fmt)); // single digit day
    }

    // Tests for DD/MM/YYYY format (UK/EU)
    #[test]
    fn valid_date_prefixes_uk() {
        let fmt = "DD/MM/YYYY";
        assert!(is_valid_date_prefix("1", fmt));
        assert!(is_valid_date_prefix("15", fmt));
        assert!(is_valid_date_prefix("15/", fmt));
        assert!(is_valid_date_prefix("15/0", fmt));
        assert!(is_valid_date_prefix("15/01", fmt));
        assert!(is_valid_date_prefix("15/01/", fmt));
        assert!(is_valid_date_prefix("15/01/2026", fmt));
    }

    #[test]
    fn leap_year_handling_uk() {
        let fmt = "DD/MM/YYYY";
        assert!(is_valid_date_prefix("29/02/2024", fmt));
        assert!(!is_valid_date_prefix("29/02/2025", fmt));
    }

    // Tests for DD.MM.YYYY format (German)
    #[test]
    fn valid_date_prefixes_german() {
        let fmt = "DD.MM.YYYY";
        assert!(is_valid_date_prefix("15", fmt));
        assert!(is_valid_date_prefix("15.", fmt));
        assert!(is_valid_date_prefix("15.01", fmt));
        assert!(is_valid_date_prefix("15.01.", fmt));
        assert!(is_valid_date_prefix("15.01.2026", fmt));
    }

    #[test]
    fn rejects_wrong_separator() {
        let fmt = "MM/DD/YYYY";
        assert!(!is_valid_date_prefix("01-15-2026", fmt)); // wrong separator
    }

    // Tests for append_date_char with auto-separator insertion
    #[test]
    fn append_date_char_iso_auto_separator() {
        let fmt = "YYYY-MM-DD";

        // After complete year, auto-insert separator
        assert_eq!(
            append_date_char("2025", '0', fmt),
            Some("2025-0".to_string())
        );

        // Normal append within component
        assert_eq!(
            append_date_char("2025-0", '1', fmt),
            Some("2025-01".to_string())
        );

        // After complete month, auto-insert separator
        assert_eq!(
            append_date_char("2025-01", '1', fmt),
            Some("2025-01-1".to_string())
        );

        // Normal append for day
        assert_eq!(
            append_date_char("2025-01-1", '5', fmt),
            Some("2025-01-15".to_string())
        );
    }

    #[test]
    fn append_date_char_us_auto_separator() {
        let fmt = "MM/DD/YYYY";

        // After complete month, auto-insert separator
        assert_eq!(append_date_char("01", '1', fmt), Some("01/1".to_string()));

        // After complete day, auto-insert separator
        assert_eq!(
            append_date_char("01/15", '2', fmt),
            Some("01/15/2".to_string())
        );

        // Normal year append
        assert_eq!(
            append_date_char("01/15/202", '5', fmt),
            Some("01/15/2025".to_string())
        );
    }

    #[test]
    fn append_date_char_german_auto_separator() {
        let fmt = "DD.MM.YYYY";

        // After complete day, auto-insert separator
        assert_eq!(append_date_char("15", '0', fmt), Some("15.0".to_string()));

        // After complete month, auto-insert separator
        assert_eq!(
            append_date_char("15.01", '2', fmt),
            Some("15.01.2".to_string())
        );
    }

    #[test]
    fn append_date_char_explicit_separator_still_works() {
        let fmt = "YYYY-MM-DD";

        // User can still type separator explicitly
        assert_eq!(
            append_date_char("2025", '-', fmt),
            Some("2025-".to_string())
        );
        assert_eq!(
            append_date_char("2025-01", '-', fmt),
            Some("2025-01-".to_string())
        );
    }

    #[test]
    fn append_date_char_rejects_invalid() {
        let fmt = "YYYY-MM-DD";

        // Can't add more digits after complete date
        assert_eq!(append_date_char("2025-01-15", '0', fmt), None);

        // Invalid month digit
        assert_eq!(append_date_char("2025-1", '9', fmt), None); // would be 19, invalid month
    }
}
