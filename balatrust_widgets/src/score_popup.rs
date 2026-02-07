use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;

use crate::theme::Theme;

/// A floating text popup that appears above a card/joker during scoring.
/// Shows text like "+10", "+4 Mult", "X2" in the appropriate color.
pub struct ScorePopup {
    pub text: String,
    pub kind: ScorePopupKind,
    /// Vertical offset from the anchor point (negative = upward float)
    pub float_offset: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScorePopupKind {
    Chips,
    Mult,
    XMult,
}

impl ScorePopup {
    pub fn new(text: impl Into<String>, kind: ScorePopupKind) -> Self {
        Self {
            text: text.into(),
            kind,
            float_offset: 0,
        }
    }

    pub fn chips(text: impl Into<String>) -> Self {
        Self::new(text, ScorePopupKind::Chips)
    }

    pub fn mult(text: impl Into<String>) -> Self {
        Self::new(text, ScorePopupKind::Mult)
    }

    pub fn xmult(text: impl Into<String>) -> Self {
        Self::new(text, ScorePopupKind::XMult)
    }

    pub fn float_offset(mut self, offset: i16) -> Self {
        self.float_offset = offset;
        self
    }

    fn color(&self) -> ratatui::style::Color {
        match self.kind {
            ScorePopupKind::Chips => Theme::CHIPS_COLOR,
            ScorePopupKind::Mult => Theme::MULT_COLOR,
            ScorePopupKind::XMult => Theme::XMULT_COLOR,
        }
    }
}

impl Widget for ScorePopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let style = Style::default()
            .fg(self.color())
            .add_modifier(Modifier::BOLD);

        // Center the text horizontally above the given area
        let text_len = self.text.len() as u16;
        let x = area.x + area.width.saturating_sub(text_len) / 2;

        // Position above the area, adjusted by float offset
        let base_y = area.y as i16 - 1 + self.float_offset;
        if base_y < 0 {
            return;
        }
        let y = base_y as u16;

        // Render background (dim the cell behind text for readability)
        for dx in 0..text_len {
            let cx = x + dx;
            if cx < buf.area().right() && y < buf.area().bottom() && y >= buf.area().top() {
                if let Some(cell) = buf.cell_mut((cx, y)) {
                    cell.set_symbol(" ");
                    cell.set_bg(Theme::PANEL_BG);
                }
            }
        }

        // Render text
        if y < buf.area().bottom() && y >= buf.area().top() {
            buf.set_string(
                x.min(buf.area().right().saturating_sub(1)),
                y,
                &self.text,
                style,
            );
        }
    }
}
