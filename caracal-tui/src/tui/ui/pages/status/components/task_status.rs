use std::time::SystemTime;

use caracal_base::{ext::ProgressChunks, model};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::tui::{
    state_store::{Action, State},
    ui::components::{Component, ComponentRender},
};

struct Props {
    /// List of rooms and current state of those rooms
    task_statuses: Vec<model::TaskStatus>,
}

impl From<&State> for Props {
    fn from(state: &State) -> Self { Self { task_statuses: state.task_statuses().clone() } }
}

pub struct TaskStatusList {
    /// Sending actions to the state store
    action_tx: UnboundedSender<Action>,
    /// State Mapped TaskStatus Props
    props: Props,
    // Internal Component State
    /// Table with optional selection and current offset
    pub table_state: TableState,
}

impl TaskStatusList {
    fn next(&mut self) {
        if self.task_statuses().is_empty() {
            self.table_state.select(None);
            return;
        }

        let i = self.table_state.selected().map_or(0, |i| {
            if i >= self.task_statuses().len() - 1 {
                0
            } else {
                i + 1
            }
        });
        self.table_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.task_statuses().is_empty() {
            self.table_state.select(None);
            return;
        }

        let i = self.table_state.selected().map_or(0, |i| {
            if i == 0 {
                self.task_statuses().len() - 1
            } else {
                i - 1
            }
        });
        self.table_state.select(Some(i));
    }

    pub(super) const fn task_statuses(&self) -> &Vec<model::TaskStatus> {
        &self.props.task_statuses
    }

    pub fn get_selected_task_status(&self) -> Option<&model::TaskStatus> {
        self.table_state.selected().and_then(|selected_idx| self.task_statuses().get(selected_idx))
    }
}

impl Component for TaskStatusList {
    fn new(state: &State, action_tx: UnboundedSender<Action>) -> Self {
        Self { action_tx, props: Props::from(state), table_state: TableState::default() }
    }

    fn move_with_state(self, state: &State) -> Self
    where
        Self: Sized,
    {
        Self { props: Props::from(state), ..self }
    }

    fn name(&self) -> &str { "Task Status List" }

    fn handle_key_event(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.next();
            }
            KeyCode::Enter => {
                if let Some(task_status) = self.get_selected_task_status() {
                    let _unused =
                        self.action_tx.send(Action::SelectTask { task_id: task_status.id });
                }
            }
            // Force refresh
            KeyCode::Char('R') => {
                let _unused = self.action_tx.send(Action::GetAllTaskStatuses);
            }
            // Delete task
            KeyCode::Delete | KeyCode::Char('d')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                if let Some(task_status) = self.get_selected_task_status() {
                    let _unused =
                        self.action_tx.send(Action::RemoveTask { task_id: task_status.id });
                }
            }
            // Pause task
            KeyCode::Char('p') => {
                if let Some(task_status) = self.get_selected_task_status() {
                    let _unused =
                        self.action_tx.send(Action::PauseTask { task_id: task_status.id });
                }
            }
            // Resume task
            KeyCode::Char('r') => {
                if let Some(task_status) = self.get_selected_task_status() {
                    let _unused =
                        self.action_tx.send(Action::ResumeTask { task_id: task_status.id });
                }
            }
            KeyCode::Char('+') => {
                if let Some(task_status) = self.get_selected_task_status() {
                    let _unused = self
                        .action_tx
                        .send(Action::IncreaseConcurrentNumber { task_id: task_status.id });
                }
            }
            KeyCode::Char('-') => {
                if let Some(task_status) = self.get_selected_task_status() {
                    let _unused = self
                        .action_tx
                        .send(Action::DecreaseConcurrentNumber { task_id: task_status.id });
                }
            }
            _ => (),
        }
    }
}

pub struct RenderProps {
    pub area: Rect,
}

impl ComponentRender<RenderProps> for TaskStatusList {
    // SAFETY: the precision loss is acceptable
    #[allow(clippy::cast_precision_loss)]
    fn render(&self, frame: &mut Frame<'_>, props: RenderProps) {
        let rects = Layout::default().constraints([Constraint::Percentage(100)]).split(props.area);

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default();
        let header_cells = [
            "ID",
            "STATE",
            "FILE NAME",
            "FILE SIZE",
            "REMAIN",
            "PROGRESS",
            "CONCURRENT",
            "PRIORITY",
            "CREATED",
        ]
        .iter()
        .map(|&h| Cell::from(h).style(Style::default()));
        let header = Row::new(header_cells).style(normal_style).height(1);
        let rows = self.task_statuses().iter().map(|status| {
            let received_bytes = status.chunks.received_bytes();
            let total_bytes = {
                let v = status.chunks.total_bytes();
                if v < received_bytes {
                    received_bytes
                } else {
                    v
                }
            };
            let progress_percentage = if total_bytes == 0 {
                if received_bytes == 0 {
                    String::from("0.00%")
                } else {
                    String::from("100.00%")
                }
            } else {
                format!("{:.2}%", (received_bytes as f64 / total_bytes as f64) * 100.0)
            };

            let cells = vec![
                Cell::from(status.id.to_string()),
                Cell::from(status.state.to_string()),
                Cell::from(status.file_path.file_name().unwrap_or_default().to_string_lossy()),
                Cell::new(humansize::format_size(total_bytes, humansize::BINARY)),
                Cell::new(humansize::format_size(
                    if received_bytes >= total_bytes { 0 } else { total_bytes - received_bytes },
                    humansize::BINARY,
                )),
                Cell::new(progress_percentage),
                Cell::new(status.concurrent_number.to_string()),
                Cell::new(status.priority.to_string()),
                Cell::new(
                    humantime::format_rfc3339_seconds(SystemTime::from(status.creation_timestamp))
                        .to_string(),
                ),
            ];
            Row::new(cells).height(1)
        });
        let table = Table::new(
            rows,
            [
                Constraint::Max(10),
                Constraint::Min(20),
                Constraint::Min(50),
                Constraint::Min(15),
                Constraint::Min(15),
                Constraint::Min(10),
                Constraint::Min(10),
                Constraint::Min(10),
                Constraint::Min(20),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Tasks")
                .title_alignment(Alignment::Center),
        )
        .highlight_style(selected_style);
        let mut app_task_status_list_state = self.table_state.clone();
        frame.render_stateful_widget(table, rects[0], &mut app_task_status_list_state);
    }
}
