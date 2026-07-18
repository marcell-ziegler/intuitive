use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{Block, BorderType, Paragraph, Widget},
};

/// The right-hand detail panel.
///
/// Currently a titled placeholder; it will grow into the statblock/detail view
/// (see `docs/PLAN.md` Phase 4).
pub struct Sidebar {
    focused: bool,
}

impl Sidebar {
    pub fn new(focused: bool) -> Self {
        Self { focused }
    }
}

impl Widget for Sidebar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("")
            .block(
                Block::bordered()
                    .title("─Sidebar")
                    .border_type(BorderType::Rounded)
                    .border_style(if self.focused {
                        Color::LightYellow
                    } else {
                        Color::LightCyan
                    }),
            )
            .render(area, buf);
    }
}
