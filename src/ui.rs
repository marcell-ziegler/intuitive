use std::default;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, List, ListState, Paragraph},
};

use crate::app::App;

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(frame.area());

    let title = Paragraph::new(Text::styled(
        "Initiative tracker",
        Style::default().fg(Color::Yellow),
    ))
    .block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightBlue)),
    );
    frame.render_widget(title, main_chunks[0]);

    let creature_list_and_stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(3), Constraint::Fill(1)])
        .split(main_chunks[1]);

    let creature_list = List::new(app.creature_representations()).block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightBlue))
            .title("Creatures"),
    );
    frame.render_stateful_widget(
        creature_list,
        creature_list_and_stats_chunks[0],
        &mut app.creature_list_state,
    );

    let stats = Paragraph::new("").block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightBlue))
            .title("Stats"),
    );

    frame.render_widget(stats, creature_list_and_stats_chunks[1]);
}
