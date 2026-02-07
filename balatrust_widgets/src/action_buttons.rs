use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;

use crate::theme::Theme;

/// Action buttons widget matching Balatro's bottom button row:
/// [Play Hand] [Sort: Rank | Suit] [Discard]
pub struct ActionButtonsWidget {
    pub can_play: bool,
    pub can_discard: bool,
    pub hands_remaining: u8,
    pub discards_remaining: u8,
}

/// Identifies which button was clicked
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonHit {
    PlayHand,
    SortRank,
    SortSuit,
    Discard,
}

impl ActionButtonsWidget {
    pub fn new(
        can_play: bool,
        can_discard: bool,
        hands_remaining: u8,
        discards_remaining: u8,
    ) -> Self {
        Self {
            can_play,
            can_discard,
            hands_remaining,
            discards_remaining,
        }
    }

    /// Get the rects for each button given the widget area.
    /// Returns (play_rect, sort_rank_rect, sort_suit_rect, discard_rect)
    pub fn button_rects(area: Rect) -> (Rect, Rect, Rect, Rect) {
        let chunks = Layout::horizontal([
            Constraint::Length(2),   // left padding
            Constraint::Ratio(1, 3), // Play Hand
            Constraint::Length(1),   // gap
            Constraint::Ratio(1, 3), // Sort area
            Constraint::Length(1),   // gap
            Constraint::Ratio(1, 3), // Discard
            Constraint::Length(2),   // right padding
        ])
        .split(area);

        let play_rect = chunks[1];
        let discard_rect = chunks[5];

        // Split sort area into two sub-buttons
        let sort_halves =
            Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).split(chunks[3]);

        (play_rect, sort_halves[0], sort_halves[1], discard_rect)
    }

    /// Check if a click at (col, row) hits any button
    pub fn hit_test(area: Rect, col: u16, row: u16) -> Option<ButtonHit> {
        let (play, sort_rank, sort_suit, discard) = Self::button_rects(area);

        if Self::point_in_rect(col, row, play) {
            return Some(ButtonHit::PlayHand);
        }
        if Self::point_in_rect(col, row, sort_rank) {
            return Some(ButtonHit::SortRank);
        }
        if Self::point_in_rect(col, row, sort_suit) {
            return Some(ButtonHit::SortSuit);
        }
        if Self::point_in_rect(col, row, discard) {
            return Some(ButtonHit::Discard);
        }
        None
    }

    fn point_in_rect(col: u16, row: u16, rect: Rect) -> bool {
        col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
    }
}

impl Widget for ActionButtonsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 3 || area.width < 20 {
            return;
        }

        let (play_rect, sort_rank_rect, sort_suit_rect, discard_rect) = Self::button_rects(area);

        // === Play Hand button ===
        render_button(
            buf,
            play_rect,
            &format!("Play Hand ({})", self.hands_remaining),
            self.can_play,
            Theme::CHIPS_COLOR,
            Theme::DIM_TEXT,
        );

        // === Sort Rank button ===
        render_button(buf, sort_rank_rect, "Rank", true, Theme::GOLD, Theme::GOLD);

        // === Sort Suit button ===
        render_button(buf, sort_suit_rect, "Suit", true, Theme::GOLD, Theme::GOLD);

        // === Discard button ===
        render_button(
            buf,
            discard_rect,
            &format!("Discard ({})", self.discards_remaining),
            self.can_discard,
            Theme::MULT_COLOR,
            Theme::DIM_TEXT,
        );
    }
}

fn render_button(
    buf: &mut Buffer,
    area: Rect,
    label: &str,
    enabled: bool,
    active_color: ratatui::style::Color,
    inactive_color: ratatui::style::Color,
) {
    if area.width < 3 || area.height < 3 {
        return;
    }

    let color = if enabled {
        active_color
    } else {
        inactive_color
    };
    let border_style = Style::default().fg(color);
    let text_style = if enabled {
        Style::default()
            .fg(Theme::BRIGHT_TEXT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::DIM_TEXT)
    };

    // Top border
    let mut top = String::new();
    top.push('\u{256d}'); // ╭
    for _ in 1..area.width.saturating_sub(1) {
        top.push('\u{2500}'); // ─
    }
    top.push('\u{256e}'); // ╮
    buf.set_string(area.x, area.y, &top, border_style);

    // Middle row (content)
    if area.height >= 2 {
        let y = area.y + 1;
        buf.set_string(area.x, y, "\u{2502}", border_style); // │
                                                             // Clear interior
        let inner_w = area.width.saturating_sub(2) as usize;
        let padding: String = " ".repeat(inner_w);
        buf.set_string(area.x + 1, y, &padding, Style::default());
        // Center label
        let display_label: String = label.chars().take(inner_w).collect();
        let label_x = area.x + 1 + (inner_w as u16).saturating_sub(display_label.len() as u16) / 2;
        buf.set_string(label_x, y, &display_label, text_style);
        buf.set_string(area.x + area.width - 1, y, "\u{2502}", border_style); // │
    }

    // Bottom border
    if area.height >= 3 {
        let y = area.y + 2;
        let mut bot = String::new();
        bot.push('\u{2570}'); // ╰
        for _ in 1..area.width.saturating_sub(1) {
            bot.push('\u{2500}'); // ─
        }
        bot.push('\u{256f}'); // ╯
        buf.set_string(area.x, y, &bot, border_style);
    }
}
