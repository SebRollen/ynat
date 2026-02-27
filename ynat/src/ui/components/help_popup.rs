use ratatui::{
    prelude::*,
    widgets::{List, ListItem},
    Frame,
};

use crate::ui::{layouts, screens::Screen, theme};

pub fn render_help_popup(f: &mut Frame, screen: &Screen) {
    let help_items = get_help_items(screen);

    // Use shared popup frame
    let inner = super::popup::render_popup_frame(
        f,
        f.area(),
        layouts::popup_sizes::LARGE,
        " Help (press ? or Esc to close) ",
        theme::accent_border_style(),
    );

    // Create the help list
    let items: Vec<ListItem> = help_items
        .iter()
        .map(|(key, description)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:15}", key), theme::header_style()),
                Span::raw(*description),
            ]))
        })
        .collect();

    let list = List::new(items).style(Style::default().fg(Color::White));

    f.render_widget(list, inner);
}

fn get_help_items(screen: &Screen) -> Vec<(&'static str, &'static str)> {
    let mut items = vec![];

    // Screen-specific help
    match screen {
        Screen::Budgets(..) => {
            items.push(("↑/k", "Move selection up"));
            items.push(("↓/j", "Move selection down"));
            items.push(("Enter/→/l", "Select budget and view accounts"));
            items.push(("r", "Refresh budgets"));
        }
        Screen::Accounts(state) => {
            items.push(("↑/k", "Move selection up"));
            items.push(("↓/j", "Move selection down"));
            items.push(("Enter/→/l", "View transactions for selected account"));
            items.push(("/", "Enter filter mode"));
            if state.input_mode == crate::state::InputMode::Filter {
                items.push(("Type", "Filter accounts by name, type, or balance"));
                items.push(("Enter", "Exit filter mode (keep filter active)"));
                items.push(("Esc", "Clear filter and exit filter mode"));
                items.push(("Backspace", "Delete last character"));
            }
            items.push((".", "Toggle showing deleted/closed accounts"));
            items.push(("r", "Refresh accounts"));
        }
        Screen::Transactions(state) => {
            items.push(("↑/k", "Move selection up"));
            items.push(("↓/j", "Move selection down"));
            items.push(("n", "Create a new transaction"));
            items.push(("e", "Edit selected transaction"));
            items.push(("a", "Approve transaction"));
            items.push(("c", "Toggle cleared status (uncleared ↔ cleared)"));
            items.push(("d/Backspace", "Delete selected transaction"));
            items.push(("/", "Enter filter mode"));
            if state.input_mode == crate::state::InputMode::Filter {
                items.push(("Type", "Filter by payee, category, memo, or amount"));
                items.push(("Enter", "Exit filter mode (keep filter active)"));
                items.push(("Esc", "Clear filter and exit filter mode"));
                items.push(("Backspace", "Delete last character"));
            }
            items.push((".", "Toggle showing reconciled transactions"));
            items.push(("r", "Refresh transactions"));
            items.push(("R", "Reconcile transactions"));
        }
        Screen::Plan(..) => {
            items.push(("↑/k", "Move selection up"));
            items.push(("↓/j", "Move selection down"));
            items.push(("e", "Edit budgeted amount"));
            items.push(("r", "Refresh plan"));
            items.push((",", "Toggle focus view"));
        }
        Screen::Logs(..) => {
            items.push(("↑/k", "Scroll up (older logs)"));
            items.push(("↓/j", "Scroll down (newer logs)"));
            items.push(("Page Up", "Scroll up one page"));
            items.push(("Page Down", "Scroll down one page"));
            items.push(("g then g", "Scroll to oldest logs"));
            items.push(("G", "Scroll to newest logs"));
        }
    }

    // Global help
    items.push(("", ""));
    items.push(("--- Global ---", ""));
    items.push(("h/←", "Navigate back"));
    items.push(("g then b", "Go to budgets"));
    items.push(("g then p", "Go to plan"));
    items.push(("g then l", "Go to logs"));
    items.push(("g then g", "Navigate to top of list"));
    items.push(("G", "Navigate to bottom of list"));
    items.push(("?", "Toggle this help"));
    items.push(("q", "Quit application"));

    items
}
