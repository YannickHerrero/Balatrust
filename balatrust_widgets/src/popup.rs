use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Widget};

use crate::theme::Theme;

/// A centered popup overlay
pub struct PopupWidget {
    pub title: String,
    pub lines: Vec<(String, Style)>,
    pub width_percent: u16,
    pub height_percent: u16,
}

impl PopupWidget {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
            width_percent: 60,
            height_percent: 40,
        }
    }

    pub fn line(mut self, text: impl Into<String>, style: Style) -> Self {
        self.lines.push((text.into(), style));
        self
    }

    pub fn size(mut self, width_percent: u16, height_percent: u16) -> Self {
        self.width_percent = width_percent;
        self.height_percent = height_percent;
        self
    }
}

impl Widget for PopupWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_width = area.width * self.width_percent / 100;
        let popup_height = area.height * self.height_percent / 100;
        let popup_x = area.x + (area.width - popup_width) / 2;
        let popup_y = area.y + (area.height - popup_height) / 2;

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Clear the area
        Clear.render(popup_area, buf);

        // Draw the popup box
        let title_line = Line::from(Span::styled(
            format!(" {} ", self.title),
            Style::default()
                .fg(Theme::GOLD)
                .add_modifier(Modifier::BOLD),
        ));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Theme::CARD_SELECTED))
            .title(title_line)
            .title_alignment(Alignment::Center)
            .padding(Padding::uniform(1));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        // Render lines centered
        for (i, (text, style)) in self.lines.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            let x = inner.x + inner.width.saturating_sub(text.len() as u16) / 2;
            buf.set_string(x, y, text, *style);
        }
    }
}

/// Helper to create a centered rect
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let width = area.width * percent_x / 100;
    let height = area.height * percent_y / 100;
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    Rect::new(x, y, width, height)
}
