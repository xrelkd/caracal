use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::Paragraph, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::tui::{
    state_store::{Action, State},
    ui::components::{Component, ComponentRender},
};

struct Props {
    server_endpoint: http::Uri,

    daemon_version: Option<semver::Version>,
}

impl From<&State> for Props {
    fn from(state: &State) -> Self {
        Self {
            server_endpoint: state.server_endpoint().clone(),
            daemon_version: state.daemon_version().map(Clone::clone),
        }
    }
}

impl Props {
    const fn is_connected(&self) -> bool { self.daemon_version.is_some() }
}

pub struct InformationArea {
    props: Props,
}

impl Component for InformationArea {
    fn new(state: &State, _action_tx: UnboundedSender<Action>) -> Self {
        Self { props: Props::from(state) }
    }

    fn move_with_state(self, state: &State) -> Self
    where
        Self: Sized,
    {
        Self { props: Props::from(state) }
    }

    fn name(&self) -> &str { "Information" }

    fn handle_key_event(&mut self, _key: KeyEvent) {}
}

pub struct RenderProps {
    pub area: Rect,
}

impl ComponentRender<RenderProps> for InformationArea {
    fn render(&self, frame: &mut Frame<'_>, props: RenderProps) {
        let [info_keys_container, info_values_container, keybind_keys_container, keybind_values_container] =
            *Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(30),
                        Constraint::Max(10),
                        Constraint::Percentage(15),
                    ]
                    .as_ref(),
                )
                .split(props.area)
        else {
            panic!("The main layout should have 5 chunks")
        };

        let info_keys = Paragraph::new(Text::from(vec![
            Line::from("TUI version:"),
            Line::from("Server version:"),
            Line::from("Connection state:"),
            Line::from("Server endpoint:"),
        ]))
        .yellow();
        frame.render_widget(info_keys, info_keys_container);

        let info_values = Paragraph::new(Text::from(vec![
            Line::from(format!(
                "{} v{}",
                caracal_base::TUI_PROGRAM_NAME,
                *caracal_base::PROJECT_SEMVER
            )),
            Line::from(self.props.daemon_version.as_ref().map_or_else(
                || "N/A".to_string(),
                |version| format!("{} v{version}", caracal_base::DAEMON_PROGRAM_NAME),
            )),
            Line::from(if self.props.is_connected() { "Connected" } else { "Disconnected" }),
            Line::from(self.props.server_endpoint.to_string()),
        ]));
        frame.render_widget(info_values, info_values_container);

        let keybind_keys = Paragraph::new(Text::from(vec![
            Line::from("<R>"),
            Line::from("<p>"),
            Line::from("<r>"),
            Line::from("<ctrl-d>"),
            Line::from("<+>"),
            Line::from("<->"),
            Line::from("<q>"),
        ]))
        .cyan();
        frame.render_widget(keybind_keys, keybind_keys_container);

        let keybind_values = Paragraph::new(Text::from(vec![
            Line::from("Refresh"),
            Line::from("Pause task"),
            Line::from("Resume task"),
            Line::from("Delete task"),
            Line::from("Increase concurrent number"),
            Line::from("Decrease concurrent number"),
            Line::from("Quit"),
        ]));
        frame.render_widget(keybind_values, keybind_values_container);
    }
}
