mod components;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use tokio::sync::mpsc;

use self::components::{
    information, information::InformationArea, task_status, task_status::TaskStatusList,
};
use crate::tui::{
    state_store::{Action, State},
    ui::components::{Component, ComponentRender},
};

pub struct StatusPage {
    action_tx: mpsc::UnboundedSender<Action>,

    information_area: InformationArea,

    task_status_list: TaskStatusList,
}

impl Component for StatusPage {
    fn new(state: &State, action_tx: mpsc::UnboundedSender<Action>) -> Self
    where
        Self: Sized,
    {
        Self {
            action_tx: action_tx.clone(),
            information_area: InformationArea::new(state, action_tx.clone()),
            task_status_list: TaskStatusList::new(state, action_tx),
        }
        .move_with_state(state)
    }

    fn move_with_state(self, state: &State) -> Self
    where
        Self: Sized,
    {
        Self {
            // propagate the update to the child components
            information_area: self.information_area.move_with_state(state),
            task_status_list: self.task_status_list.move_with_state(state),
            ..self
        }
    }

    fn name(&self) -> &str { "Status Page" }

    fn handle_key_event(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                let _unused = self.action_tx.send(Action::Shutdown);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let _unused = self.action_tx.send(Action::Shutdown);
            }
            KeyCode::Char('a') => {
                // TODO:
                // show popup for creating new task
            }
            _ => {
                self.task_status_list.handle_key_event(key);
            }
        }
    }
}

impl ComponentRender<()> for StatusPage {
    fn render(&self, frame: &mut Frame<'_>, _props: ()) {
        let [information_area, task_status_list_area] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(20), Constraint::Percentage(80)].as_ref())
            .split(frame.size())
        else {
            panic!("The main layout should have 2 chunks")
        };

        self.information_area.render(frame, information::RenderProps { area: information_area });
        self.task_status_list
            .render(frame, task_status::RenderProps { area: task_status_list_area });
    }
}
