use ynab_api::endpoints::{categories::Category, payees::Payee};

/// Filter payees by query string for autocomplete
/// Returns up to 10 matching payees
pub fn filter_payees(payees: &[Payee], query: &str) -> Vec<Payee> {
    if query.is_empty() {
        return payees.iter().take(10).cloned().collect();
    }

    let query_lower = query.to_lowercase();
    payees
        .iter()
        .filter(|p| p.name.to_lowercase().contains(&query_lower))
        .take(10)
        .cloned()
        .collect()
}

/// Filter categories by query string for autocomplete
/// Returns up to 10 matching categories
/// Matches against both category name and "Group: Category" format
pub fn filter_categories(categories: &[Category], query: &str) -> Vec<Category> {
    if query.is_empty() {
        return categories.iter().take(10).cloned().collect();
    }

    let query_lower = query.to_lowercase();
    categories
        .iter()
        .filter(|c| {
            let full_name = if let Some(ref group_name) = c.category_group_name {
                format!("{}: {}", group_name, c.name)
            } else {
                c.name.clone()
            };
            full_name.to_lowercase().contains(&query_lower)
                || c.name.to_lowercase().contains(&query_lower)
        })
        .take(10)
        .cloned()
        .collect()
}
