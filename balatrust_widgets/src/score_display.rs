use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Widget};

use crate::theme::Theme;

/// Score display panel showing chips x mult and round progress
pub struct ScoreDisplayWidget {
    pub hand_name: String,
    pub hand_level: u8,
    pub chips: u64,
    pub mult: u64,
    pub round_score: u64,
    pub score_target: u64,
}

impl ScoreDisplayWidget {
    pub fn new(
        hand_name: String,
        hand_level: u8,
        chips: u64,
        mult: u64,
        round_score: u64,
        score_target: u64,
    ) -> Self {
        Self {
            hand_name,
            hand_level,
            chips,
            mult,
            round_score,
            score_target,
        }
    }
}

impl Widget for ScoreDisplayWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Theme::CARD_BORDER))
            .padding(Padding::horizontal(1));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 4 {
            return;
        }

        let mut y = inner.y;

        // Hand name + level
        if !self.hand_name.is_empty() {
            let hand_line = Line::from(vec![
                Span::styled(
                    &self.hand_name,
                    Style::default()
                        .fg(Theme::BRIGHT_TEXT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" Lv.{}", self.hand_level),
                    Style::default().fg(Theme::GOLD),
                ),
            ]);
            buf.set_line(inner.x, y, &hand_line, inner.width);
            y += 1;
        }

        // Blank line
        y += 1;

        // Chips x Mult
        let score_line = Line::from(vec![
            Span::styled(
                format!("{}", self.chips),
                Style::default()
                    .fg(Theme::CHIPS_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " \u{00d7} ",
                Style::default()
                    .fg(Theme::BRIGHT_TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", self.mult),
                Style::default()
                    .fg(Theme::MULT_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        buf.set_line(inner.x, y, &score_line, inner.width);
        y += 2;

        // Round score / target
        if y < inner.bottom() {
            let progress_line = Line::from(vec![
                Span::styled("Score: ", Style::default().fg(Theme::MUTED_TEXT)),
                Span::styled(
                    format_number(self.round_score),
                    Style::default()
                        .fg(Theme::SCORE_COLOR)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            buf.set_line(inner.x, y, &progress_line, inner.width);
            y += 1;
        }

        if y < inner.bottom() {
            let target_line = Line::from(vec![
                Span::styled("Target: ", Style::default().fg(Theme::MUTED_TEXT)),
                Span::styled(
                    format_number(self.score_target),
                    Style::default().fg(Theme::BRIGHT_TEXT),
                ),
            ]);
            buf.set_line(inner.x, y, &target_line, inner.width);
            y += 1;
        }

        // Progress bar
        if y < inner.bottom() && self.score_target > 0 {
            let progress = (self.round_score as f64 / self.score_target as f64).min(1.0);
            let bar_width = inner.width.saturating_sub(2) as f64;
            let filled = (bar_width * progress) as u16;

            let mut bar = String::new();
            bar.push('[');
            for i in 0..bar_width as u16 {
                if i < filled {
                    bar.push('\u{2588}'); // █
                } else {
                    bar.push('\u{2591}'); // ░
                }
            }
            bar.push(']');

            let bar_color = if progress >= 1.0 {
                Theme::MONEY_COLOR
            } else {
                Theme::CHIPS_COLOR
            };
            let bar_style = Style::default().fg(bar_color);
            buf.set_string(inner.x, y, &bar, bar_style);
        }
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        // Add comma separator
        let s = n.to_string();
        let mut result = String::new();
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        result.chars().rev().collect()
    } else {
        n.to_string()
    }
}
