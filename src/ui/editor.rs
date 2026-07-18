use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Clear, Paragraph, Widget},
};
use tui_input::Input;

use crate::app::{EditorField, EditorState};

/// The modal creature editor.
///
/// Rendered as an overlay centered within the area it is given (the full frame).
/// Borrows the [`EditorState`] read-only; text buffers live in `EditorState`.
pub struct Editor<'a> {
    state: &'a EditorState,
}

impl<'a> Editor<'a> {
    pub fn new(state: &'a EditorState) -> Self {
        Self { state }
    }
}

impl Widget for Editor<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let editor_area = centered_rect_fixed_height(40, 2 + 3 * 5, area);
        let input_chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(
            Layout::horizontal([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(editor_area)[1],
        );

        Clear.render(editor_area, buf);
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::LightBlue)
            .title("Edit Creature")
            .render(editor_area, buf);

        let s = self.state;
        render_input(
            &s.name_input,
            "Name",
            s.active_input == EditorField::Name,
            input_chunks[1],
            buf,
        );

        let hp_chunks = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .split(input_chunks[2]);
        render_input(
            &s.cur_hp_input,
            "Current HP",
            s.active_input == EditorField::CurrentHP,
            hp_chunks[0],
            buf,
        );
        Paragraph::new("   \n / \n   ")
            .bold()
            .white()
            .render(hp_chunks[1], buf);
        render_input(
            &s.max_hp_input,
            "Max HP",
            s.active_input == EditorField::MaxHP,
            hp_chunks[2],
            buf,
        );
        render_input(
            &s.ac_input,
            "AC",
            s.active_input == EditorField::AC,
            input_chunks[3],
            buf,
        );
        render_input(
            &s.cr_input,
            "Lvl / CR",
            s.active_input == EditorField::CR,
            input_chunks[4],
            buf,
        );
        render_input(
            &s.amount_input,
            "Amount",
            s.active_input == EditorField::Amount,
            input_chunks[5],
            buf,
        );
    }
}

/// Render a single bordered text field, scrolled to keep the tail visible.
fn render_input(input: &Input, name: &str, active: bool, area: Rect, buf: &mut Buffer) {
    // keep 2 for borders and 1 for cursor
    let width = area.width.max(3) - 3;
    let scroll = input.visual_scroll(width as usize);
    let style: Style = if active {
        Color::Yellow.into()
    } else {
        Color::White.into()
    };
    Paragraph::new(input.value())
        .style(style)
        .scroll((0, scroll as u16))
        .block(Block::bordered().title(name))
        .render(area, buf);
}

/// Return a centered `Rect` area with width as a percentage and height in lines.
fn centered_rect_fixed_height(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Length((area.height.saturating_sub(height)) / 2),
        Constraint::Length(height),
        Constraint::Min(0),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

/// Return a centered `Rect` area, sized as a percentage of `area` in both axes.
///
/// Unused scaffolding for future modals (kept intentionally; see CLAUDE.md).
#[allow(dead_code)]
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
