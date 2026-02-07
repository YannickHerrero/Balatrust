use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::theme::Theme;

/// Bottom HUD showing hands, discards, money, deck count
pub struct HudWidget {
    pub hands: u8,
    pub discards: u8,
    pub money: u32,
    pub deck_remaining: usize,
    pub can_play: bool,
    pub can_discard: bool,
}

impl HudWidget {
    pub fn new(hands: u8, discards: u8, money: u32, deck_remaining: usize) -> Self {
        Self {
            hands,
            discards,
            money,
            deck_remaining,
            can_play: false,
            can_discard: false,
        }
    }

    pub fn can_play(mut self, can_play: bool) -> Self {
        self.can_play = can_play;
        self
    }

    pub fn can_discard(mut self, can_discard: bool) -> Self {
        self.can_discard = can_discard;
        self
    }
}

impl Widget for HudWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        let chunks = Layout::horizontal([
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
        ])
        .split(area);

        // Play Hand button
        let play_style = if self.can_play {
            Style::default()
                .fg(Theme::BRIGHT_TEXT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::DIM_TEXT)
        };
        let play_line = Line::from(vec![
            Span::styled(
                "[P]",
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("lay Hand", play_style),
        ]);
        buf.set_line(chunks[0].x, chunks[0].y, &play_line, chunks[0].width);

        // Discard button
        let disc_style = if self.can_discard {
            Style::default()
                .fg(Theme::BRIGHT_TEXT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::DIM_TEXT)
        };
        let disc_line = Line::from(vec![
            Span::styled(
                "[D]",
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("iscard", disc_style),
        ]);
        buf.set_line(chunks[1].x, chunks[1].y, &disc_line, chunks[1].width);

        // Hands remaining
        let hands_line = Line::from(vec![
            Span::styled("Hands: ", Style::default().fg(Theme::MUTED_TEXT)),
            Span::styled(
                format!("{}", self.hands),
                Style::default()
                    .fg(Theme::CHIPS_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        buf.set_line(chunks[2].x, chunks[2].y, &hands_line, chunks[2].width);

        // Discards remaining
        let discs_line = Line::from(vec![
            Span::styled("Discards: ", Style::default().fg(Theme::MUTED_TEXT)),
            Span::styled(
                format!("{}", self.discards),
                Style::default()
                    .fg(Theme::MULT_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        buf.set_line(chunks[3].x, chunks[3].y, &discs_line, chunks[3].width);

        // Money & Deck
        let money_line = Line::from(vec![
            Span::styled(
                format!("${}", self.money),
                Style::default()
                    .fg(Theme::MONEY_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Deck:", Style::default().fg(Theme::MUTED_TEXT)),
            Span::styled(
                format!("{}", self.deck_remaining),
                Style::default().fg(Theme::BRIGHT_TEXT),
            ),
        ]);
        buf.set_line(chunks[4].x, chunks[4].y, &money_line, chunks[4].width);
    }
}
