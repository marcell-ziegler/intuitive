use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Padding, Row, StatefulWidget, Table, TableState},
};

use crate::storage::Encounter;

/// The initiative-order table.
///
/// Borrows the [`Encounter`] it renders so a fresh widget is cheaply built each
/// frame. Selection/scroll are driven through the [`TableState`] passed as the
/// widget state, so the widget itself stays read-only over the model.
pub struct InitiativeTable<'a> {
    encounter: &'a Encounter,
    focused: bool,
}

impl<'a> InitiativeTable<'a> {
    pub fn new(encounter: &'a Encounter, focused: bool) -> Self {
        Self { encounter, focused }
    }
}

impl StatefulWidget for InitiativeTable<'_> {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header = Row::new(["Name", "Lvl", "HP", "AC", "Initiative"])
            .bold()
            .bottom_margin(1);

        let mut rows = Vec::new();
        for (i, creature) in self.encounter.creatures.iter().enumerate() {
            let is_selected = self.encounter.cursor_index == i;
            let is_initiative = self.encounter.initiative_index == i;

            let (icon, row_style) = match (is_selected, is_initiative) {
                (true, true) => ("󰞇", Style::new().on_yellow().dark_gray()),
                (true, false) => (" ", Style::new().on_dark_gray()),
                (false, true) => ("󰞇 ", Style::new().on_yellow().dark_gray()),
                (false, false) => ("  ", Style::default()),
            };

            rows.push(
                Row::new([
                    format!("{}{}", icon, creature.name()),
                    format_level_or_cr(creature.get_level_or_cr()),
                    format!("{}/{}", creature.hp(), creature.max_hp()),
                    creature.ac().to_string(),
                    match creature.get_initiative() {
                        Some(i) => i.to_string(),
                        None => String::from("n/a"),
                    },
                ])
                .style(row_style),
            );
        }

        let table = Table::new(
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
                .title_bottom(keybind_hint())
                .border_type(BorderType::Rounded)
                .border_style(if self.focused {
                    Color::LightYellow
                } else {
                    Color::LightCyan
                })
                .padding(Padding::symmetric(1, 0)),
        );

        StatefulWidget::render(table, area, buf, state);
    }
}

/// Format a level (integer) or challenge rating (may be fractional) for display.
fn format_level_or_cr(value: f64) -> String {
    if value.fract() <= f64::EPSILON {
        value.floor().to_string()
    } else {
        match value {
            0.25 => String::from("1/4"),
            0.5 => String::from("1/2"),
            0.75 => String::from("3/4"),
            _ => value.floor().to_string(),
        }
    }
}

/// The keybind hint rendered along the bottom border of the table.
fn keybind_hint() -> Line<'static> {
    Line::from(
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
    )
}
