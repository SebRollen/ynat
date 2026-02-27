use crate::state::TransactionFormState;
use crate::ui::utils as ui_utils;
use chrono::NaiveDate;
use uuid::Uuid;
use ynab_api::endpoints::{
    categories::Category,
    payees::Payee,
    transactions::{NewSubTransaction, NewTransaction, TransactionUpdate},
};

/// Validate and build a NewTransaction from form state
pub fn validate_and_build_transaction(
    form: &TransactionFormState,
    payees: &[Payee],
    categories: &[Category],
    date_format: &str,
) -> Result<NewTransaction, String> {
    // Check if this is a split transaction
    if form.is_split_mode {
        return validate_and_build_split_transaction(form, payees, categories, date_format);
    }

    // Validate date and convert to ISO format
    let date = validate_date(&form.date, date_format)?;

    // Validate amount
    let amount_milliunits = validate_amount(&form.amount)?;

    // Resolve payee (ID or name)
    let (payee_id, payee_name) = resolve_payee(&form.payee, payees);

    // Resolve category
    let category_id = resolve_category(&form.category, categories);

    // Parse account_id
    let account_uuid =
        Uuid::parse_str(&form.account_id).map_err(|_| "Invalid account ID".to_string())?;

    // Build transaction
    Ok(NewTransaction {
        account_id: account_uuid,
        date,
        amount: amount_milliunits.into(),
        payee_id,
        payee_name,
        category_id,
        memo: if form.memo.is_empty() {
            None
        } else {
            Some(form.memo.clone())
        },
        cleared: Some(form.cleared),
        approved: Some(true),
        flag_color: form.flag_color,
        subtransactions: None,
    })
}

/// Validate and build a split transaction (transaction with subtransactions)
fn validate_and_build_split_transaction(
    form: &TransactionFormState,
    payees: &[Payee],
    categories: &[Category],
    date_format: &str,
) -> Result<NewTransaction, String> {
    // Validate date and convert to ISO format
    let date = validate_date(&form.date, date_format)?;

    // Validate parent amount
    let parent_amount = validate_amount(&form.amount)?;

    // Resolve payee (ID or name)
    let (payee_id, payee_name) = resolve_payee(&form.payee, payees);

    // Parse account_id
    let account_uuid =
        Uuid::parse_str(&form.account_id).map_err(|_| "Invalid account ID".to_string())?;

    // Validate each subtransaction
    let mut subtransactions = Vec::new();
    let mut sum_of_subtransactions: i64 = 0;

    for (i, sub) in form.subtransactions.iter().enumerate() {
        let sub_amount =
            validate_amount(&sub.amount).map_err(|e| format!("Split {}: {}", i + 1, e))?;
        sum_of_subtransactions += sub_amount;

        let category_id = resolve_category(&sub.category, categories);

        subtransactions.push(NewSubTransaction {
            amount: sub_amount.into(),
            category_id,
            memo: if sub.memo.is_empty() {
                None
            } else {
                Some(sub.memo.clone())
            },
        });
    }

    // Verify subtransaction amounts sum equals parent amount
    if sum_of_subtransactions != parent_amount {
        let diff = parent_amount - sum_of_subtransactions;
        let diff_formatted = format!("{:.2}", diff.abs() as f64 / 1000.0);
        if diff > 0 {
            return Err(format!(
                "Split amounts are ${} under the total",
                diff_formatted
            ));
        } else {
            return Err(format!(
                "Split amounts are ${} over the total",
                diff_formatted
            ));
        }
    }

    // Build transaction with subtransactions
    Ok(NewTransaction {
        account_id: account_uuid,
        date,
        amount: parent_amount.into(),
        payee_id,
        payee_name,
        category_id: None, // Split transactions don't have a parent category
        memo: if form.memo.is_empty() {
            None
        } else {
            Some(form.memo.clone())
        },
        cleared: Some(form.cleared),
        approved: Some(true),
        flag_color: form.flag_color,
        subtransactions: Some(subtransactions),
    })
}

fn validate_date(date_str: &str, date_format: &str) -> Result<String, String> {
    // Convert from user format to ISO format
    let iso_date = ui_utils::parse_user_date(date_str, date_format)?;

    // Validate the converted date
    NaiveDate::parse_from_str(&iso_date, "%Y-%m-%d")
        .map(|_| iso_date)
        .map_err(|_| format!("Invalid date. Use format: {}", date_format))
}

fn validate_date_as_naive(date_str: &str, date_format: &str) -> Result<NaiveDate, String> {
    // Convert from user format to ISO format
    let iso_date = ui_utils::parse_user_date(date_str, date_format)?;

    // Parse and return the NaiveDate
    NaiveDate::parse_from_str(&iso_date, "%Y-%m-%d")
        .map_err(|_| format!("Invalid date. Use format: {}", date_format))
}

fn validate_amount(amount_str: &str) -> Result<i64, String> {
    if amount_str.is_empty() {
        return Err("Amount cannot be empty".to_string());
    }

    let amount: f64 = amount_str.parse().map_err(|_| {
        "Invalid amount. Enter a number (e.g., -50.00 for outflow, 50.00 for inflow)".to_string()
    })?;

    // Convert to milliunits (YNAB uses milliunits: 1000 = $1.00)
    Ok((amount * 1000.0) as i64)
}

fn resolve_payee(input: &str, payees: &[Payee]) -> (Option<Uuid>, Option<String>) {
    if input.is_empty() {
        return (None, None);
    }

    // Exact match by name (case-insensitive)
    if let Some(payee) = payees.iter().find(|p| p.name.eq_ignore_ascii_case(input)) {
        (Some(payee.id), None)
    } else {
        // No match, send as payee_name (YNAB will create new payee)
        (None, Some(input.to_string()))
    }
}

fn resolve_category(input: &str, categories: &[Category]) -> Option<Uuid> {
    if input.is_empty() {
        return None;
    }

    // Match by full name with group prefix or just category name
    categories
        .iter()
        .find(|c| {
            let full_name = if let Some(ref group_name) = c.category_group_name {
                format!("{}: {}", group_name, c.name)
            } else {
                c.name.clone()
            };
            full_name.eq_ignore_ascii_case(input) || c.name.eq_ignore_ascii_case(input)
        })
        .map(|c| c.id)
}

/// Build a TransactionUpdate from form state (for editing)
pub fn build_transaction_update(
    form: &TransactionFormState,
    payees: &[Payee],
    categories: &[Category],
    date_format: &str,
) -> Result<TransactionUpdate, String> {
    // Validate date and convert to NaiveDate
    let date = validate_date_as_naive(&form.date, date_format)?;

    // Validate amount
    let amount_milliunits = validate_amount(&form.amount)?;

    // Resolve payee
    let (payee_id, payee_name) = resolve_payee(&form.payee, payees);

    // Handle split transactions
    let (category_id, subtransactions) = if form.is_split_mode {
        // Validate subtransactions
        let mut subs = Vec::new();
        let mut sum_of_subtransactions: i64 = 0;

        for (i, sub) in form.subtransactions.iter().enumerate() {
            let sub_amount =
                validate_amount(&sub.amount).map_err(|e| format!("Split {}: {}", i + 1, e))?;
            sum_of_subtransactions += sub_amount;

            let cat_id = resolve_category(&sub.category, categories);

            subs.push(NewSubTransaction {
                amount: sub_amount.into(),
                category_id: cat_id,
                memo: if sub.memo.is_empty() {
                    None
                } else {
                    Some(sub.memo.clone())
                },
            });
        }

        // Verify subtransaction amounts sum equals parent amount
        if sum_of_subtransactions != amount_milliunits {
            let diff = amount_milliunits - sum_of_subtransactions;
            let diff_formatted = format!("{:.2}", diff.abs() as f64 / 1000.0);
            if diff > 0 {
                return Err(format!(
                    "Split amounts are ${} under the total",
                    diff_formatted
                ));
            } else {
                return Err(format!(
                    "Split amounts are ${} over the total",
                    diff_formatted
                ));
            }
        }

        (None, Some(subs)) // Split transactions don't have a parent category
    } else {
        // Regular transaction
        (resolve_category(&form.category, categories), None)
    };

    Ok(TransactionUpdate {
        account_id: None, // Don't change account when editing
        date: Some(date),
        amount: Some(amount_milliunits.into()),
        payee_id,
        payee_name,
        category_id,
        memo: if form.memo.is_empty() {
            None
        } else {
            Some(form.memo.clone())
        },
        flag_color: form.flag_color,
        cleared: Some(form.cleared),
        approved: Some(true),
        subtransactions,
    })
}
