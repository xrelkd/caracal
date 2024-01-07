use std::{io, time::Duration};

use crossterm::event::{Event, EventStream};
use futures::FutureExt;
use ratatui::{prelude::CrosstermBackend, terminal::Terminal};
use snafu::ResultExt;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::tui::{
    state_store::Action,
    ui::{
        components::{Component, ComponentRender},
        error,
        pages::AppRouter,
        Error,
    },
    State,
};

const RENDERING_TICK_RATE: Duration = Duration::from_millis(250);

pub struct UiManager {
    action_tx: mpsc::UnboundedSender<Action>,
}

impl UiManager {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Action>) {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        (Self { action_tx }, action_rx)
    }

    pub async fn serve(
        self,
        mut state_rx: mpsc::UnboundedReceiver<State>,
        shutdown: sigfinn::Shutdown,
    ) -> Result<(), Error> {
        // consume the first state to initialize the ui app
        let mut app_router = {
            let state = state_rx.recv().await.expect("state_tx must not be empty");

            AppRouter::new(&state, self.action_tx.clone())
        };

        let mut terminal = setup_terminal()?;
        let mut ticker = tokio::time::interval(RENDERING_TICK_RATE);
        let mut crossterm_events = EventStream::new();
        let mut shutdown = shutdown.into_stream();

        let result = loop {
            tokio::select! {
                // Tick to terminate the select every N milliseconds
                _ = ticker.tick() => (),
                // Catch and handle crossterm events
                maybe_event = crossterm_events.next() => match maybe_event {
                    Some(Ok(Event::Key(key)))  => {
                        app_router.handle_key_event(key);
                    },
                    None => break Ok(()),
                    _ => (),
                },
                // Handle state updates
                Some(state) = state_rx.recv() => {
                    app_router = app_router.move_with_state(&state);
                },
                 // Catch and handle interrupt signal to gracefully shutdown
                Some(()) = shutdown.next() => {
                    break Ok(());
                }
            }

            if let Err(err) = terminal
                .draw(|frame| app_router.render(frame, ()))
                .context(error::RenderTerminalSnafu)
            {
                break Err(err);
            }
        };

        restore_terminal(&mut terminal)?;

        result
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Error> {
    let mut stdout = io::stdout();

    crossterm::terminal::enable_raw_mode().context(error::EnableRawModeSnafu)?;

    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )
    .context(error::EnterAlternateScreenSnafu)?;

    Terminal::new(CrosstermBackend::new(stdout)).context(error::CreateTerminalSnafu)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Error> {
    crossterm::terminal::disable_raw_mode().context(error::DisableRawModeSnafu)?;

    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )
    .context(error::EnterMainScreenSnafu)?;

    terminal.show_cursor().context(error::ShowCursorSnafu)
}
