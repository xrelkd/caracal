use crossterm::event::KeyEvent;
use ratatui::Frame;
use tokio::sync::mpsc;

use crate::tui::state_store::{Action, State};

pub trait Component {
    fn new(state: &State, action_tx: mpsc::UnboundedSender<Action>) -> Self
    where
        Self: Sized;
    fn move_with_state(self, state: &State) -> Self
    where
        Self: Sized;

    fn name(&self) -> &str;

    fn handle_key_event(&mut self, key: KeyEvent);
}

pub trait ComponentRender<Props> {
    fn render(&self, frame: &mut Frame<'_>, props: Props);
}
