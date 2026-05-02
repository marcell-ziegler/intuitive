use std::ops::Add;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Padding, Paragraph, Row, Table},
};

use crate::app::{App, Panel};

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    // Main UI Chunks, header and main space
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

    // Main content chunks, table and sidebar
    let content_chunks = Layout::horizontal([Constraint::Min(25), Constraint::Length(36)])
        .spacing(1)
        .split(chunks[1]);

    // Sidebar
    let sidebar_placeholder = Paragraph::new("").block(
        Block::bordered()
            .title("─Sidebar")
            .border_type(BorderType::Rounded)
            .border_style(if app.current_panel == Panel::Sidebar {
                Color::LightYellow
            } else {
                Color::LightCyan
            }),
    );

    frame.render_widget(sidebar_placeholder, content_chunks[1]);

    // Main table
    render_initiative_table(frame, app, content_chunks[0]);

    if app.current_panel == Panel::Editor {
        render_editor(frame, app)
    }
}

fn render_editor(frame: &mut Frame, app: &mut App) {
    let editor_area = centered_rect(60, 40, frame.area());
    
}

fn render_initiative_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let header = Row::new(["Name", "Lvl", "HP", "AC", "Initiative"])
        .bold()
        .bottom_margin(1);

    app.sync_table_state();

    let mut rows = Vec::new();
    for (i, creature) in app.current_encounter.creatures.iter().enumerate() {
        let is_selected = app.current_encounter.cursor_index == i;
        let is_initiative = app.current_encounter.initiative_index == i;

        let (icon, row_style) = match (is_selected, is_initiative) {
            (true, true) => ("󰞇", Style::new().on_yellow().dark_gray()),
            (true, false) => (" ", Style::new().on_dark_gray()),
            (false, true) => ("󰞇 ", Style::new().on_yellow().dark_gray()),
            (false, false) => ("  ", Style::default()),
        };

        rows.push(
            Row::new([
                format!("{}{}", icon, creature.name()),
                if creature.get_level_or_cr().fract() <= f64::EPSILON {
                    creature.get_level_or_cr().floor().to_string()
                } else {
                    match creature.get_level_or_cr() {
                        0.25 => String::from("1/4"),
                        0.5 => String::from("1/2"),
                        0.75 => String::from("3/4"),
                        _ => creature.get_level_or_cr().floor().to_string(),
                    }
                },
                format!("{}/{}", creature.hp(), creature.max_hp()),
                creature.ac().to_string(),
                match creature.get_initiative() {
                    Some(i) => i.to_string(),
                    None => String::from("n/a"),
                },
            ])
            .style(row_style),
        )
    }

    let tab = Table::new(
        rows,
        [
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .column_spacing(1)
    .block(
        Block::bordered()
            .title("─Initiative Order")
            .title_bottom(Line::from(
                Span::from("─")
                    + Span::from("k/j").bold().white()
                    + Span::from("─")
                    + Span::from("Up/Down").white()
                    + Span::from("──")
                    + Span::from("Tab").bold().white()
                    + Span::from("─")
                    + Span::from("Swap Panel").white()
                    + Span::from("──")
                    + Span::from("n").bold().white()
                    + Span::from("─")
                    + Span::from("Add Creature").white()
                    + Span::from("──"),
            ))
            .border_type(BorderType::Rounded)
            .border_style(if app.current_panel == Panel::InitiativeTable {
                Color::LightYellow
            } else {
                Color::LightCyan
            })
            .padding(Padding::symmetric(1, 0)),
    );

    frame.render_stateful_widget(tab, area, &mut app.main_table_state);
}

/// Return a centered `Rect` area.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
