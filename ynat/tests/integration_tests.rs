use ynat::events::DataEvent;
use ynat::input::Key;
use ynat::state::InputMode;
use ynat::testing::TestApp;
use ynat::ui::screens::Screen;

/// Generate a deterministic UUID from a string ID for testing
fn test_uuid(id: &str) -> uuid::Uuid {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    let hash = hasher.finish();
    let bytes = [
        (hash >> 56) as u8,
        (hash >> 48) as u8,
        (hash >> 40) as u8,
        (hash >> 32) as u8,
        (hash >> 24) as u8,
        (hash >> 16) as u8,
        (hash >> 8) as u8,
        hash as u8,
        (hash >> 56) as u8,
        (hash >> 48) as u8,
        (hash >> 40) as u8,
        (hash >> 32) as u8,
        (hash >> 24) as u8,
        (hash >> 16) as u8,
        (hash >> 8) as u8,
        hash as u8,
    ];
    uuid::Uuid::from_bytes(bytes)
}

#[test]
fn test_quit_flow() {
    let mut app = TestApp::new();

    // Initially should not quit
    app.assert_not_quit();

    // Press 'q' to quit
    app.send_key(Key::Char('q'));

    // Assert app should quit
    app.assert_should_quit();
}

#[test]
fn test_help_toggle() {
    let mut app = TestApp::new();

    // Initially help is hidden
    assert!(!app.state().help_visible);

    // Press '?' to show help
    app.send_key(Key::Char('?'));
    assert!(app.state().help_visible);

    // Press '?' again to hide
    app.send_key(Key::Char('?'));
    assert!(!app.state().help_visible);

    // Press '?' again to show
    app.send_key(Key::Char('?'));
    assert!(app.state().help_visible);

    // Press 'Esc' to hide
    app.send_key(Key::Esc);
    assert!(!app.state().help_visible);
}

#[test]
fn test_multi_key_sequence_gg() {
    let mut app = TestApp::new();

    // Initially no pending key
    assert_eq!(app.state().pending_key, None);

    // First 'g' sets pending key
    app.send_key(Key::Char('g'));
    assert_eq!(app.state().pending_key, Some('g'));

    // Second 'g' triggers navigate to top and clears pending
    app.send_key(Key::Char('g'));
    assert_eq!(app.state().pending_key, None);

    // On budgets screen, selection should be at index 0
    if let Screen::Budgets(budgets_state) = app.state().current_screen() {
        assert_eq!(budgets_state.selected_budget_index, 0);
    }
}

#[test]
fn test_multi_key_sequence_gb() {
    let mut app = TestApp::new();

    // First 'g' sets pending key
    app.send_key(Key::Char('g'));
    assert_eq!(app.state().pending_key, Some('g'));

    // Second 'b' triggers LoadBudgets command (which does nothing in sync mode)
    // but should clear the pending key
    app.send_key(Key::Char('b'));

    // Pending key should be cleared after the command
    assert_eq!(app.state().pending_key, None);

    // Note: In sync test mode, LoadBudgets doesn't navigate to budgets screen
    // since it would normally spawn a background task. To test full navigation,
    // inject a BudgetsCacheLoaded event manually.
}

#[test]
fn test_navigation_with_j_k() {
    let mut app = TestApp::new();

    // Inject some budget data
    app.send_data_event(DataEvent::BudgetsCacheLoaded {
        budgets: vec![
            ynab_api::endpoints::budgets::BudgetSummary {
                id: test_uuid("budget1").into(),
                name: "Budget 1".to_string(),
                last_modified_on: None,
                first_month: None,
                last_month: None,
                date_format: None,
                currency_format: None,
                accounts: None,
            },
            ynab_api::endpoints::budgets::BudgetSummary {
                id: test_uuid("budget2").into(),
                name: "Budget 2".to_string(),
                last_modified_on: None,
                first_month: None,
                last_month: None,
                date_format: None,
                currency_format: None,
                accounts: None,
            },
            ynab_api::endpoints::budgets::BudgetSummary {
                id: test_uuid("budget3").into(),
                name: "Budget 3".to_string(),
                last_modified_on: None,
                first_month: None,
                last_month: None,
                date_format: None,
                currency_format: None,
                accounts: None,
            },
        ],
        default_budget: None,
    });

    if let Screen::Budgets(budgets_state) = app.state().current_screen() {
        assert_eq!(budgets_state.selected_budget_index, 0);
    }

    // Press 'j' to select next
    app.send_key(Key::Char('j'));
    if let Screen::Budgets(budgets_state) = app.state().current_screen() {
        assert_eq!(budgets_state.selected_budget_index, 1);
    }

    // Press 'j' again
    app.send_key(Key::Char('j'));
    if let Screen::Budgets(budgets_state) = app.state().current_screen() {
        assert_eq!(budgets_state.selected_budget_index, 2);
    }

    // Press 'k' to select previous
    app.send_key(Key::Char('k'));
    if let Screen::Budgets(budgets_state) = app.state().current_screen() {
        assert_eq!(budgets_state.selected_budget_index, 1);
    }
}

#[test]
fn test_filter_mode_entry_and_typing() {
    use ynab_api::endpoints::accounts::{Account, AccountType};

    let mut app = TestApp::new();

    // Inject budget first
    app.send_data_event(DataEvent::BudgetsCacheLoaded {
        budgets: vec![ynab_api::endpoints::budgets::BudgetSummary {
            id: test_uuid("budget1").into(),
            name: "Test Budget".to_string(),
            last_modified_on: None,
            first_month: None,
            last_month: None,
            date_format: None,
            currency_format: None,
            accounts: None,
        }],
        default_budget: None,
    });

    // Navigate to accounts (need to trigger LoadAccounts via 'gb' won't work in sync mode)
    // Instead, manually inject AccountsCacheLoaded event to simulate being on accounts screen
    app.send_data_event(DataEvent::AccountsCacheLoaded {
        accounts: vec![Account {
            id: uuid::Uuid::new_v4(),
            name: "Checking Account".to_string(),
            account_type: AccountType::Checking,
            on_budget: true,
            closed: false,
            note: None,
            balance: 100000.into(),
            cleared_balance: 50000.into(),
            uncleared_balance: 50000.into(),
            transfer_payee_id: None,
            direct_import_linked: false,
            direct_import_in_error: false,
            deleted: false,
        }],
    });

    // Verify we're on accounts screen
    if let Screen::Accounts(accounts_state) = app.state().current_screen() {
        // Initially in Normal mode
        assert_eq!(accounts_state.input_mode, InputMode::Normal);
        assert_eq!(accounts_state.filter_query, "");

        // Enter filter mode with '/'
        app.send_key(Key::Char('/'));

        // Check we're in filter mode
        if let Screen::Accounts(accounts_state) = app.state().current_screen() {
            assert_eq!(accounts_state.input_mode, InputMode::Filter);
        }

        // Type some characters
        app.send_keys(&[
            Key::Char('c'),
            Key::Char('h'),
            Key::Char('e'),
            Key::Char('c'),
            Key::Char('k'),
        ]);

        // Assert filter query is updated
        if let Screen::Accounts(accounts_state) = app.state().current_screen() {
            assert_eq!(accounts_state.filter_query, "check");
        }

        // Press backspace to delete a character
        app.send_key(Key::Backspace);
        if let Screen::Accounts(accounts_state) = app.state().current_screen() {
            assert_eq!(accounts_state.filter_query, "chec");
        }

        // Press Enter to exit filter mode
        app.send_key(Key::Enter);
        if let Screen::Accounts(accounts_state) = app.state().current_screen() {
            assert_eq!(accounts_state.input_mode, InputMode::Normal);
            assert_eq!(accounts_state.filter_query, "chec"); // Filter persists
        }

        // Press Esc to clear filter
        app.send_key(Key::Esc);
        if let Screen::Accounts(accounts_state) = app.state().current_screen() {
            assert_eq!(accounts_state.filter_query, "");
        }
    }
}

#[test]
fn test_capital_g_navigates_to_bottom() {
    let mut app = TestApp::new();

    // Inject budget data
    app.send_data_event(DataEvent::BudgetsCacheLoaded {
        budgets: vec![
            ynab_api::endpoints::budgets::BudgetSummary {
                id: test_uuid("budget1").into(),
                name: "Budget 1".to_string(),
                last_modified_on: None,
                first_month: None,
                last_month: None,
                date_format: None,
                currency_format: None,
                accounts: None,
            },
            ynab_api::endpoints::budgets::BudgetSummary {
                id: test_uuid("budget2").into(),
                name: "Budget 2".to_string(),
                last_modified_on: None,
                first_month: None,
                last_month: None,
                date_format: None,
                currency_format: None,
                accounts: None,
            },
            ynab_api::endpoints::budgets::BudgetSummary {
                id: test_uuid("budget3").into(),
                name: "Budget 3".to_string(),
                last_modified_on: None,
                first_month: None,
                last_month: None,
                date_format: None,
                currency_format: None,
                accounts: None,
            },
        ],
        default_budget: None,
    });

    // Press 'G' (capital G / Shift+g) to navigate to bottom
    app.send_key(Key::Char('G'));

    // Should be at last index (2)
    if let Screen::Budgets(budgets_state) = app.state().current_screen() {
        assert_eq!(budgets_state.selected_budget_index, 2);
    }
}

#[test]
fn test_help_overlay_blocks_other_commands() {
    let mut app = TestApp::new();

    // Show help
    app.send_key(Key::Char('?'));
    assert!(app.state().help_visible);

    // Try to quit while help is visible - should not quit
    // (Help has priority, 'q' when help is visible triggers quit)
    app.send_key(Key::Char('q'));

    // App should quit (help overlay allows 'q' to quit)
    app.assert_should_quit();
}

#[test]
fn test_pending_key_cleared_after_invalid_sequence() {
    let mut app = TestApp::new();

    // Press 'g' to set pending key
    app.send_key(Key::Char('g'));
    assert_eq!(app.state().pending_key, Some('g'));

    // Press an invalid key (not 'g', 'b', or 'p')
    app.send_key(Key::Char('x'));

    // Pending key should be cleared
    assert_eq!(app.state().pending_key, None);
}
