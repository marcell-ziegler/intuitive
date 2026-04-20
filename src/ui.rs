use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Padding, Paragraph, Row, Table},
};

use crate::app::App;

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    // Main UI Chunks, header and main space
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(frame.area());

    let title = Paragraph::new(Span::styled(
        "Intuitive --- Initiative Tracker",
        Style::default().italic().fg(Color::LightBlue),
    ))
    .block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightCyan)),
    );

    frame.render_widget(title, chunks[0]);

    // Main content chunks, table and sidebar
    let content_chunks =
        Layout::horizontal([Constraint::Min(25), Constraint::Length(16)]).split(chunks[1]);

    // Sidebar
    let sidebar_placeholder = Paragraph::new("Sidebar").block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightCyan)),
    );

    frame.render_widget(sidebar_placeholder, content_chunks[1]);

    // Main table
    render_initiative_table(frame, app, content_chunks[0]);
}

fn render_initiative_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let header = Row::new(["Name", "Lvl", "HP", "AC", "Initiative"])
        .bold()
        .bottom_margin(1);

    let mut rows = Vec::new();
    for creature in app.creatures.iter() {
        rows.push(Row::new([
            creature.name().to_string(),
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
        ]))
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
            .border_type(BorderType::Rounded)
            .border_style(Color::LightCyan)
            .padding(Padding::symmetric(1, 0)),
    )
    .highlight_symbol("󰞇 ")
    .row_highlight_style(Style::new().bold().on_yellow());

    frame.render_stateful_widget(tab, area, &mut app.main_table_state);
}
