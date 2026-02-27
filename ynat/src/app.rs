use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::sync::Arc;
use ynat_auth::StoredToken;

use crate::background::{data_loader::DataLoader, BackgroundTaskManager};
use crate::cache::Cache;
use crate::commands::{executor, handlers};
use crate::input::KeyEvent;
use crate::log_buffer::LogBuffer;
use crate::logging::init_logging_with_buffer;
use crate::state::AppState;
use crate::ui::screens::Screen;
use ynab_api::Client;

pub struct App {
    token: StoredToken,
}

impl App {
    pub fn new(token: StoredToken) -> Self {
        Self { token }
    }

    pub async fn run(&self) -> Result<()> {
        // Create log buffer before initializing logging
        let log_buffer = LogBuffer::new(5000);
        let _log_path = init_logging_with_buffer(log_buffer.clone())?;

        tracing::info!("ynat starting");

        let mut terminal = self.init()?;
        let cache = Arc::new(Cache::new().await?);

        let (data_tx, mut data_rx) = tokio::sync::mpsc::unbounded_channel();

        let mut ui_state = AppState::new();
        let mut task_manager = BackgroundTaskManager::new();

        let api_client = Arc::new(Client::new(&self.token.access_token));
        let data_loader = DataLoader::new(api_client.clone(), cache.clone(), data_tx.clone());

        let mut event_stream = EventStream::new();

        self.init_data(&mut ui_state, &mut task_manager, &data_loader);

        tracing::info!("Entering main event loop");

        let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
        loop {
            // Update total_entries for logs screen if active
            if let Screen::Logs(logs_state) = ui_state.current_screen_mut() {
                logs_state.total_entries = log_buffer.len();
            }

            terminal.draw(|f| {
                crate::ui::render_app(f, &ui_state, &log_buffer);
            })?;

            tokio::select! {
                _ = interval.tick() => {
                    if let Some(throbber_state) = ui_state.loading_state() {
                        throbber_state.calc_next();
                    }
                }
                Some(Ok(event)) = event_stream.next() => {
                    match event {
                        Event::Key(key) if matches!(key.kind, KeyEventKind::Press) => {
                            // Don't log when on logs screen to avoid feedback loop
                            let on_logs_screen = matches!(ui_state.current_screen(), Screen::Logs(_));
                            if !on_logs_screen {
                                tracing::debug!("Key press: {:?}", key);
                            }
                            if let Some(command) = handlers::handle_key_input(KeyEvent::from(key), &ui_state) {
                                if !on_logs_screen {
                                    tracing::info!("Executing command: {:?}", command);
                                }
                                executor::execute_command(
                                    command,
                                    &mut ui_state,
                                    &mut task_manager,
                                    &data_loader,
                                );
                            }
                        }
                        _ => {
                            // Ignore other events
                        }
                    }
                }
                Some(data_event) = data_rx.recv() => {
                    tracing::debug!("Received data event: {:?}", data_event);
                    crate::state::reducer::reduce_data_event(&mut ui_state, data_event);
                }
            }

            // Check if we should quit
            if ui_state.should_quit {
                tracing::info!("Quit requested, exiting event loop");
                break;
            }
        }

        tracing::info!("Cleaning up application");

        // Cancel all background data loading tasks
        task_manager.cancel_all();

        self.exit(terminal)?;

        Ok(())
    }

    fn init(&self) -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, std::io::Error> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        Terminal::new(backend)
    }

    fn init_data(
        &self,
        ui_state: &mut AppState,
        task_manager: &mut BackgroundTaskManager,
        data_loader: &DataLoader,
    ) {
        tracing::info!("Loading default budget accounts");
        executor::execute_command(
            crate::commands::AppCommand::LoadBudgets {
                force_refresh: false,
                load_accounts: false,
            },
            ui_state,
            task_manager,
            data_loader,
        );
        executor::execute_command(
            crate::commands::AppCommand::LoadAccounts {
                budget_id: "default".to_string(),
                budget: Box::new(None),
                force_refresh: false,
            },
            ui_state,
            task_manager,
            data_loader,
        );
        executor::execute_command(
            crate::commands::AppCommand::LoadPayees {
                budget_id: "default".to_string(),
            },
            ui_state,
            task_manager,
            data_loader,
        );
        executor::execute_command(
            crate::commands::AppCommand::LoadCategories {
                budget_id: "default".to_string(),
            },
            ui_state,
            task_manager,
            data_loader,
        );
    }

    fn exit(
        &self,
        mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), std::io::Error> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }
}
