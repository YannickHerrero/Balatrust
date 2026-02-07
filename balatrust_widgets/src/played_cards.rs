use balatrust_core::card::PlayingCard;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use crate::card::{CardWidget, CARD_HEIGHT, CARD_WIDTH};

/// Widget that renders the played cards in the scoring zone.
/// During scoring animation, individual cards can be highlighted.
pub struct PlayedCardsWidget<'a> {
    pub cards: &'a [PlayingCard],
    /// Which card indices are part of the scoring hand
    pub scoring_indices: &'a [usize],
    /// The index of the card currently being scored (gets bright glow)
    pub active_card: Option<usize>,
    pub spacing: u16,
}

impl<'a> PlayedCardsWidget<'a> {
    pub fn new(cards: &'a [PlayingCard], scoring_indices: &'a [usize]) -> Self {
        Self {
            cards,
            scoring_indices,
            active_card: None,
            spacing: 2,
        }
    }

    pub fn active_card(mut self, active: Option<usize>) -> Self {
        self.active_card = active;
        self
    }

    /// Calculate the total width needed
    pub fn total_width(&self) -> u16 {
        let n = self.cards.len() as u16;
        if n == 0 {
            return 0;
        }
        n * CARD_WIDTH + (n - 1) * self.spacing
    }

    /// Get the Rect for a specific card given the area
    pub fn card_rect(&self, area: Rect, card_index: usize) -> Option<Rect> {
        if card_index >= self.cards.len() {
            return None;
        }

        let total_w = self.total_width();
        let start_x = area.x + area.width.saturating_sub(total_w) / 2;
        let x = start_x + (card_index as u16) * (CARD_WIDTH + self.spacing);
        let y = area.y;

        Some(Rect::new(x, y, CARD_WIDTH, CARD_HEIGHT))
    }
}

impl<'a> Widget for PlayedCardsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < CARD_HEIGHT || self.cards.is_empty() {
            return;
        }

        for (i, card) in self.cards.iter().enumerate() {
            if let Some(card_area) = self.card_rect(area, i) {
                if card_area.right() > area.right() || card_area.bottom() > area.bottom() {
                    continue;
                }

                let is_scoring_card = self.scoring_indices.contains(&i);
                let is_active = self.active_card == Some(i);

                CardWidget::new(*card)
                    .scoring(is_active)
                    .dimmed(!is_scoring_card && !is_active)
                    .render(card_area, buf);
            }
        }
    }
}
