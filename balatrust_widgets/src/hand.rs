use balatrust_core::card::PlayingCard;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use crate::card::{CardWidget, CARD_HEIGHT, CARD_WIDTH};

/// Widget that renders a fan of cards (the player's hand).
/// Selected cards are rendered one row higher to show "raised" effect.
pub struct HandWidget<'a> {
    pub cards: &'a [PlayingCard],
    pub selected_indices: &'a [usize],
    pub cursor: Option<usize>,
    pub spacing: u16,
}

impl<'a> HandWidget<'a> {
    pub fn new(cards: &'a [PlayingCard], selected_indices: &'a [usize]) -> Self {
        Self {
            cards,
            selected_indices,
            cursor: None,
            spacing: 1,
        }
    }

    pub fn cursor(mut self, cursor: Option<usize>) -> Self {
        self.cursor = cursor;
        self
    }

    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Calculate the total width needed for the hand
    pub fn total_width(&self) -> u16 {
        let n = self.cards.len() as u16;
        if n == 0 {
            return 0;
        }
        n * CARD_WIDTH + (n - 1) * self.spacing
    }

    /// Get the Rect for a specific card given the hand area
    pub fn card_rect(&self, area: Rect, card_index: usize) -> Option<Rect> {
        if card_index >= self.cards.len() {
            return None;
        }

        let total_w = self.total_width();
        let start_x = area.x + area.width.saturating_sub(total_w) / 2;

        let x = start_x + (card_index as u16) * (CARD_WIDTH + self.spacing);
        let is_selected = self.selected_indices.contains(&card_index);

        // Selected cards render 1 row higher
        let y = if is_selected { area.y } else { area.y + 1 };

        Some(Rect::new(x, y, CARD_WIDTH, CARD_HEIGHT))
    }
}

impl<'a> Widget for HandWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < CARD_HEIGHT + 1 {
            return;
        }

        for (i, card) in self.cards.iter().enumerate() {
            if let Some(card_area) = self.card_rect(area, i) {
                // Bounds check
                if card_area.right() > area.right() || card_area.bottom() > area.bottom() {
                    continue;
                }

                let is_selected = self.selected_indices.contains(&i);
                let is_cursor = self.cursor == Some(i);

                CardWidget::new(*card)
                    .selected(is_selected)
                    .highlighted(is_cursor && !is_selected)
                    .dimmed(card.debuffed)
                    .render(card_area, buf);
            }
        }
    }
}
