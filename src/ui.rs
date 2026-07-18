use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, BorderType, Paragraph},
};

use crate::app::{App, Panel};

mod editor;
mod sidebar;
mod table;

use editor::Editor;
use sidebar::Sidebar;
use table::InitiativeTable;

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    // Main UI chunks: header and main space.
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(frame.area());

    let title = Paragraph::new(Span::styled(
        "Intuitive --- Initiative Tracker",
        Style::default().italic().fg(Color::Yellow),
    ))
    .block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightCyan)),
    );
    frame.render_widget(title, chunks[0]);

    // Main content chunks: table and sidebar.
    let content_chunks = Layout::horizontal([Constraint::Min(25), Constraint::Length(36)])
        .spacing(1)
        .split(chunks[1]);

    frame.render_widget(
        Sidebar::new(app.current_panel == Panel::Sidebar),
        content_chunks[1],
    );

    app.sync_table_state();
    let focused = app.current_panel == Panel::InitiativeTable;
    frame.render_stateful_widget(
        InitiativeTable::new(&app.current_encounter, focused),
        content_chunks[0],
        &mut app.main_table_state,
    );

    if app.current_panel == Panel::Editor {
        let area = frame.area();
        frame.render_widget(Editor::new(&app.editor_state), area);
    }
}
