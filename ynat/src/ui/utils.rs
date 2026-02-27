use ratatui::{style::Color, text::Span};
use ynab_api::endpoints::{
    budgets::BudgetSummary, transactions::FlagColor, CurrencyFormat, DateFormat,
};

use super::theme;

pub fn fmt_dollars(amount: f64) -> Span<'static> {
    if amount >= 0.0 {
        Span::from(format!(" ${:.2}", amount))
    } else {
        Span::from(format!("-${:.2}", amount.abs()))
    }
}

pub fn flag_color_to_ratatui_color(color: &FlagColor) -> Color {
    match color {
        FlagColor::Red => Color::Red,
        FlagColor::Blue => Color::Blue,
        FlagColor::Green => Color::Green,
        FlagColor::Orange => Color::Indexed(94),
        FlagColor::Yellow => Color::Yellow,
        FlagColor::Purple => Color::Indexed(128),
    }
}

/// Format currency using the budget's currency format
pub fn fmt_currency(amount: i64, currency_format: &CurrencyFormat) -> Span<'static> {
    // YNAB amounts are in milliunits (1000 = 1.00)
    let amount_float = amount as f64 / 1000.0;
    let is_negative = amount_float < 0.0;
    let abs_amount = amount_float.abs();

    // Format the number with the appropriate decimal digits
    let formatted_number = format_number_with_separators(
        abs_amount,
        currency_format.decimal_digits as usize,
        &currency_format.decimal_separator,
        &currency_format.group_separator,
    );

    // Build the final string based on currency format preferences
    let result = if currency_format.display_symbol {
        if currency_format.symbol_first {
            if is_negative {
                format!("-{}{}", currency_format.currency_symbol, formatted_number)
            } else {
                format!(" {}{}", currency_format.currency_symbol, formatted_number)
            }
        } else if is_negative {
            format!("-{}{}", formatted_number, currency_format.currency_symbol)
        } else {
            format!(" {}{}", formatted_number, currency_format.currency_symbol)
        }
    } else if is_negative {
        format!("-{}", formatted_number)
    } else {
        format!(" {}", formatted_number)
    };

    Span::from(result)
}

/// Format a number with thousands separators and decimal separator
fn format_number_with_separators(
    amount: f64,
    decimal_digits: usize,
    decimal_separator: &str,
    group_separator: &str,
) -> String {
    // Split into integer and decimal parts
    let integer_part = amount.floor() as i64;
    let decimal_part =
        ((amount - amount.floor()) * 10_f64.powi(decimal_digits as i32)).round() as i64;

    // Format integer part with group separators
    let integer_str = integer_part.to_string();
    let mut formatted_integer = String::new();
    for (i, c) in integer_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            formatted_integer.insert(0, group_separator.chars().next().unwrap_or(','));
        }
        formatted_integer.insert(0, c);
    }

    // Format decimal part
    if decimal_digits > 0 {
        let decimal_str = format!("{:0width$}", decimal_part, width = decimal_digits);
        format!("{}{}{}", formatted_integer, decimal_separator, decimal_str)
    } else {
        formatted_integer
    }
}

/// Format date using the budget's date format
/// Input date is expected to be in YYYY-MM-DD format (ISO 8601)
pub fn fmt_date(date_str: &str, date_format: &DateFormat) -> String {
    fmt_date_with_format(date_str, &date_format.format)
}

/// Format date using a format string
/// Input date is expected to be in YYYY-MM-DD format (ISO 8601)
pub fn fmt_date_with_format(date_str: &str, format: &str) -> String {
    // Parse the input date (YYYY-MM-DD)
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return date_str.to_string(); // Return as-is if invalid format
    }

    let year = parts[0];
    let month = parts[1];
    let day = parts[2];

    // Apply the format pattern
    // Common patterns: MM/DD/YYYY, DD/MM/YYYY, YYYY-MM-DD, etc.
    format
        .replace("YYYY", year)
        .replace("MM", month)
        .replace("DD", day)
}

/// Parse a date from user format to ISO format (YYYY-MM-DD)
/// Example: "01/15/2026" with format "MM/DD/YYYY" -> Ok("2026-01-15")
pub fn parse_user_date(date_str: &str, format: &str) -> Result<String, String> {
    // Find the separator in the format
    let separator = format
        .chars()
        .find(|c| !c.is_ascii_alphabetic())
        .ok_or("Invalid date format: no separator found")?;

    // Split the format and input by separator
    let format_parts: Vec<&str> = format.split(separator).collect();
    let date_parts: Vec<&str> = date_str.split(separator).collect();

    if format_parts.len() != 3 || date_parts.len() != 3 {
        return Err("Invalid date: expected 3 parts separated by delimiter".to_string());
    }

    let mut year = "";
    let mut month = "";
    let mut day = "";

    for (fmt_part, date_part) in format_parts.iter().zip(date_parts.iter()) {
        if fmt_part.contains("YYYY") {
            year = date_part;
        } else if fmt_part.contains("MM") {
            month = date_part;
        } else if fmt_part.contains("DD") {
            day = date_part;
        }
    }

    if year.is_empty() || month.is_empty() || day.is_empty() {
        return Err("Could not parse date components".to_string());
    }

    Ok(format!("{}-{}-{}", year, month, day))
}

/// Get the separator character from a date format string
pub fn get_date_separator(format: &str) -> char {
    format
        .chars()
        .find(|c| !c.is_ascii_alphabetic())
        .unwrap_or('-')
}

// =============================================================================
// Amount Formatting (consolidated from screens)
// =============================================================================

/// Format an amount using the budget's currency format, or fallback to dollars.
/// This consolidates the duplicate format_amount functions from screens.
pub fn format_amount(amount: i64, budget: Option<&BudgetSummary>) -> String {
    if let Some(budget) = budget {
        if let Some(ref currency_format) = budget.currency_format {
            return fmt_currency(amount, currency_format).content.into();
        }
    }
    let amount_f64 = amount as f64 / 1000.0;
    fmt_dollars(amount_f64).content.into()
}

/// Get the appropriate color for an amount value.
/// Re-exports from theme for convenience.
pub fn get_amount_color(amount: i64) -> Color {
    theme::amount_color(amount)
}

/// Get the appropriate color for a float amount value.
/// Re-exports from theme for convenience.
pub fn get_amount_color_f64(amount: f64) -> Color {
    theme::amount_color_f64(amount)
}
