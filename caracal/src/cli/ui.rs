use std::time::SystemTime;

use caracal_base::{ext::ProgressChunks, model};
use comfy_table::{Cell, ContentArrangement, Row, Table, presets::NOTHING};

// SAFETY: the precision loss is acceptable
#[allow(clippy::cast_precision_loss)]
pub fn render_task_statuses_table(task_statuses: &[model::TaskStatus]) -> String {
    let header = Row::from([
        "ID",
        "STATE",
        "FILE PATH",
        "RECEIVED",
        "SIZE",
        "PROGRESS",
        "CONCURRENT",
        "PRIORITY",
        "CREATED",
    ]);
    let rows = task_statuses.iter().map(|status| {
        let received_bytes = status.chunks.received_bytes();
        let total_bytes = {
            let v = status.chunks.total_bytes();
            if v < received_bytes { received_bytes } else { v }
        };
        let progress_percentage = if total_bytes == 0 {
            if received_bytes == 0 { String::from("0.00%") } else { String::from("100.00%") }
        } else {
            format!("{:.2}%", (received_bytes as f64 / total_bytes as f64) * 100.0)
        };
        Row::from([
            Cell::new(status.id),
            Cell::new(status.state),
            Cell::new(status.file_path.display()),
            Cell::new(humansize::format_size(received_bytes, humansize::BINARY)),
            Cell::new(humansize::format_size(total_bytes, humansize::BINARY)),
            Cell::new(progress_percentage),
            Cell::new(status.concurrent_number),
            Cell::new(status.priority),
            Cell::new(humantime::format_rfc3339_seconds(SystemTime::from(
                status.creation_timestamp,
            ))),
        ])
    });

    build_table().set_header(header).add_rows(rows).to_string()
}

pub fn build_table() -> Table {
    let mut table = Table::new();
    let _ = table.load_style(NOTHING).set_content_arrangement(ContentArrangement::Dynamic);

    if let Some(width) = table.width() {
        let _ = table.set_width(width - 10);
    }

    table
}
