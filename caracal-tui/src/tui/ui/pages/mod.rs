mod status;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use tokio::sync::mpsc;

use self::status::StatusPage;
use crate::tui::{
    state_store::{Action, State},
    ui::components::{Component, ComponentRender},
};

enum ActivePage {
    StatusPage,
}

struct Props {
    active_page: ActivePage,
}

impl From<&State> for Props {
    fn from(_state: &State) -> Self { Self { active_page: ActivePage::StatusPage } }
}

pub struct AppRouter {
    props: Props,
    status_page: StatusPage,
}

impl AppRouter {
    fn get_active_page_component(&self) -> &dyn Component {
        match self.props.active_page {
            ActivePage::StatusPage => &self.status_page,
        }
    }

    fn get_active_page_component_mut(&mut self) -> &mut dyn Component {
        match self.props.active_page {
            ActivePage::StatusPage => &mut self.status_page,
        }
    }
}

impl Component for AppRouter {
    fn new(state: &State, action_tx: mpsc::UnboundedSender<Action>) -> Self
    where
        Self: Sized,
    {
        Self { props: Props::from(state), status_page: StatusPage::new(state, action_tx) }
            .move_with_state(state)
    }

    fn move_with_state(self, state: &State) -> Self
    where
        Self: Sized,
    {
        Self { props: Props::from(state), status_page: self.status_page.move_with_state(state) }
    }

    // route all functions to the active page
    fn name(&self) -> &str { self.get_active_page_component().name() }

    fn handle_key_event(&mut self, key: KeyEvent) {
        self.get_active_page_component_mut().handle_key_event(key);
    }
}

impl ComponentRender<()> for AppRouter {
    fn render(&self, frame: &mut Frame<'_>, props: ()) {
        match self.props.active_page {
            ActivePage::StatusPage => self.status_page.render(frame, props),
        }
    }
}
