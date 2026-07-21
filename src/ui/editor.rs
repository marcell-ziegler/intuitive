use crate::editor::{CreatureType, EditorField, EditorInput, EditorState, HpMode};
use crate::model::Status;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Widget},
};

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
        // Wide enough for two columns side by side (left: the original
        // fields; right: Hit Die, HP mode, Statuses).
        let editor_area = centered_rect_fixed_height(74, 2 + 3 * 5, area);

        Clear.render(editor_area, buf);
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::LightBlue)
            .title("Edit Creature")
            .title_bottom(keybind_hint())
            .render(editor_area, buf);

        let columns = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
            .spacing(1)
            .split(
                Layout::horizontal([
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(editor_area)[1],
            );

        let input_chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(columns[0]);

        let right_chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(columns[1]);

        let s = self.state;
        render_input(
            &s.name,
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
            &s.cur_hp,
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
            &s.max_hp,
            "Max HP",
            s.active_input == EditorField::MaxHP,
            hp_chunks[2],
            buf,
        );
        render_input(
            &s.ac,
            "AC",
            s.active_input == EditorField::AC,
            input_chunks[3],
            buf,
        );
        let creature_type_cr_chunks =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(input_chunks[4]);
        render_input(
            &s.cr,
            "Lvl / CR",
            s.active_input == EditorField::CR,
            creature_type_cr_chunks[0],
            buf,
        );
        render_toggle_box(
            "Creature type (^t)",
            match s.creature_type {
                CreatureType::Monster => "Monster",
                CreatureType::Player => "Player",
            },
            match s.creature_type {
                CreatureType::Monster => Color::LightGreen.into(),
                CreatureType::Player => Color::LightBlue.into(),
            },
            creature_type_cr_chunks[1],
            buf,
        );
        let amount_initiative_chunks = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .split(input_chunks[5]);
        render_input(
            &s.amount,
            "Amount",
            s.active_input == EditorField::Amount,
            amount_initiative_chunks[0],
            buf,
        );
        Paragraph::new("   \n / \n   ")
            .bold()
            .white()
            .render(amount_initiative_chunks[1], buf);
        render_input(
            &s.initiative,
            "Initiative",
            s.active_input == EditorField::Initiative,
            amount_initiative_chunks[2],
            buf,
        );

        render_input(
            &s.hit_die,
            "Hit Die",
            s.active_input == EditorField::HitDie,
            right_chunks[1],
            buf,
        );

        let editing = s.editing_index.is_some();
        let hp_mode_label = if editing {
            "HP Mode (locked)"
        } else {
            "HP Mode (^r)"
        };
        let (hp_mode_text, hp_mode_style): (&str, Style) = match s.hp_mode {
            HpMode::Roll => ("Roll", Color::LightMagenta.into()),
            HpMode::Manual => ("Manual", Color::Gray.into()),
        };
        let hp_mode_style = if editing {
            Style::new().fg(Color::DarkGray)
        } else {
            hp_mode_style
        };
        render_toggle_box(
            hp_mode_label,
            hp_mode_text,
            hp_mode_style,
            right_chunks[2],
            buf,
        );

        render_statuses(
            s,
            s.active_input == EditorField::Statuses,
            right_chunks[3],
            buf,
        );
    }
}

/// The keybind hint rendered along the bottom border of the modal.
fn keybind_hint() -> Line<'static> {
    Span::from("─")
        + Span::from("ctrl+t").bold().white()
        + Span::from(" type").white()
        + Span::from("──")
        + Span::from("ctrl+r").bold().white()
        + Span::from(" hp mode").white()
        + Span::from("──")
        + Span::from("ctrl+enter").bold().white()
        + Span::from(" submit").white()
        + Span::from("──")
}

/// Render a single bordered text field, scrolled to keep the tail visible.
///
/// The border turns red when the field is invalid; otherwise the text is
/// highlighted yellow while the field is active.
fn render_input(field: &EditorInput, name: &str, active: bool, area: Rect, buf: &mut Buffer) {
    // keep 2 for borders and 1 for cursor
    let width = area.width.max(3) - 3;
    let scroll = field.input.visual_scroll(width as usize);
    let text_style: Style = if active {
        Color::Yellow.into()
    } else {
        Color::White.into()
    };
    let mut block = Block::bordered().title(name);
    if !field.valid {
        block = block.border_style(Style::new().fg(Color::Red));
    }
    Paragraph::new(field.input.value())
        .style(text_style)
        .scroll((0, scroll as u16))
        .block(block)
        .render(area, buf);
}

fn render_toggle_box(name: &str, text: &str, style: Style, area: Rect, buf: &mut Buffer) {
    Paragraph::new(text)
        .style(style)
        .block(Block::bordered().title(name).border_style(style))
        .render(area, buf);
}

/// Render the Statuses field: the live input on its own line, followed by
/// the running `pending_statuses` list — the entry at `selected_status_index`
/// highlighted (only meaningful while this field is focused, since that's
/// the only time Left/Right/Backspace act on the selection rather than text).
fn render_statuses(state: &EditorState, active: bool, area: Rect, buf: &mut Buffer) {
    let input_style: Style = if active {
        Color::Yellow.into()
    } else {
        Color::White.into()
    };
    let mut lines = vec![Line::styled(
        state.statuses_input.input.value(),
        input_style,
    )];

    if state.pending_statuses.is_empty() {
        lines.push(Line::styled("(none)", Style::new().fg(Color::DarkGray)));
    } else {
        for (i, status) in state.pending_statuses.iter().enumerate() {
            let selected = active && Some(i) == state.selected_status_index;
            let style = if selected {
                Style::new().on_yellow().dark_gray()
            } else {
                Style::new().white()
            };
            lines.push(Line::styled(format_status(status), style));
        }
    }

    let mut block = Block::bordered().title("Statuses (enter: add, del: remove)");
    if !state.statuses_input.valid {
        block = block.border_style(Style::new().fg(Color::Red));
    } else if active {
        block = block.border_style(Color::LightYellow);
    }

    Paragraph::new(lines).block(block).render(area, buf);
}

/// A short display label for a status chip. Unit variants (most of them) use
/// their `Debug` output as-is (e.g. `Poisoned`); `Exhaustion` shows its
/// count. `Grappled`'s target is a `CreatureId`, not a name — this widget
/// only borrows `EditorState`, not the encounter, so it can't look the name
/// back up; showing just the status name is an acceptable simplification.
fn format_status(status: &Status) -> String {
    match status {
        Status::Exhaustion(n) => format!("Exhaustion ({n})"),
        Status::Grappled(_) => "Grappled".to_string(),
        other => format!("{other:?}"),
    }
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
/// Unused scaffolding for future modals
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
