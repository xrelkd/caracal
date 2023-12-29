use caracal_base::model;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Row, Table, TableComponent};

pub fn render_task_statuses_table(task_statuses: &[model::TaskStatus]) -> String {
    let header = Row::from(["ID", "STATE", "FILE PATH", "SIZE", "CONCURRENT NUMBER", "PRIORITY"]);
    let rows = task_statuses.iter().map(|status| {
        Row::from([
            Cell::new(status.id),
            Cell::new(status.state),
            Cell::new(status.file_path.display()),
            Cell::new(status.content_length),
            Cell::new(status.concurrent_number),
            Cell::new(status.priority),
        ])
    });

    build_table().set_header(header).add_rows(rows).to_string()
}

pub fn build_table() -> Table {
    let mut table = Table::new();
    let _ = table
        .load_preset(UTF8_FULL)
        .set_style(TableComponent::MiddleHeaderIntersections, '═')
        .remove_style(TableComponent::BottomBorder)
        .remove_style(TableComponent::BottomBorderIntersections)
        .remove_style(TableComponent::BottomLeftCorner)
        .remove_style(TableComponent::BottomRightCorner)
        .remove_style(TableComponent::HorizontalLines)
        .remove_style(TableComponent::LeftBorder)
        .remove_style(TableComponent::LeftBorderIntersections)
        .remove_style(TableComponent::LeftHeaderIntersection)
        .remove_style(TableComponent::MiddleIntersections)
        .remove_style(TableComponent::RightBorder)
        .remove_style(TableComponent::RightBorderIntersections)
        .remove_style(TableComponent::RightHeaderIntersection)
        .remove_style(TableComponent::TopBorder)
        .remove_style(TableComponent::TopBorderIntersections)
        .remove_style(TableComponent::TopLeftCorner)
        .remove_style(TableComponent::TopRightCorner)
        .remove_style(TableComponent::VerticalLines)
        .set_content_arrangement(ContentArrangement::Dynamic);

    if let Some(width) = table.width() {
        let _ = table.set_width(width - 10);
    };

    table
}
