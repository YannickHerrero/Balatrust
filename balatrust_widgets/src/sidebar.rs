use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use crate::theme::Theme;

/// Left sidebar widget matching Balatro's original layout.
/// Displays blind info, score, hand type, chips x mult, game info, money, and ante/round.
pub struct SidebarWidget {
    // Blind info
    pub blind_name: String,
    pub blind_color: ratatui::style::Color,
    pub score_target: u64,
    pub reward: u32,

    // Score
    pub round_score: u64,

    // Hand type + chips/mult
    pub hand_name: String,
    pub hand_level: u8,
    pub chips: u64,
    pub mult: u64,

    // Game info
    pub hands_remaining: u8,
    pub discards_remaining: u8,

    // Money
    pub money: u32,

    // Meta
    pub ante: u8,
    pub max_ante: u8,
    pub round_number: u8,

    // Mode
    /// When true, shows a reduced sidebar (no blind banner, blind info, or hand type)
    pub recap: bool,
}

impl SidebarWidget {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        blind_name: String,
        blind_color: ratatui::style::Color,
        score_target: u64,
        reward: u32,
        round_score: u64,
        hand_name: String,
        hand_level: u8,
        chips: u64,
        mult: u64,
        hands_remaining: u8,
        discards_remaining: u8,
        money: u32,
        ante: u8,
        max_ante: u8,
        round_number: u8,
    ) -> Self {
        Self {
            blind_name,
            blind_color,
            score_target,
            reward,
            round_score,
            hand_name,
            hand_level,
            chips,
            mult,
            hands_remaining,
            discards_remaining,
            money,
            ante,
            max_ante,
            round_number,
            recap: false,
        }
    }

    /// Set recap mode (simplified sidebar for post-blind screen)
    pub fn recap(mut self, recap: bool) -> Self {
        self.recap = recap;
        self
    }
}

impl Widget for SidebarWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 10 {
            return;
        }

        // Outer border for the whole sidebar
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Theme::CARD_BORDER));

        let inner = outer_block.inner(area);
        outer_block.render(area, buf);

        if self.recap {
            // Recap mode: simplified sidebar (no blind banner, blind info, or hand type)
            let sections = Layout::vertical([
                Constraint::Length(3), // Round score
                Constraint::Length(3), // Chips x Mult display
                Constraint::Length(1), // Separator
                Constraint::Length(3), // Game info (hands + discards)
                Constraint::Length(2), // Money
                Constraint::Min(0),    // Spacer
                Constraint::Length(2), // Ante / Round meta
            ])
            .split(inner);

            self.render_round_score(sections[0], buf);
            self.render_chips_mult(sections[1], buf);
            self.render_separator(sections[2], buf);
            self.render_game_info(sections[3], buf);
            self.render_money(sections[4], buf);
            self.render_meta(sections[6], buf);
        } else {
            // Normal mode: full sidebar
            let sections = Layout::vertical([
                Constraint::Length(3), // Blind name banner
                Constraint::Length(4), // Blind info (target + reward)
                Constraint::Length(3), // Round score
                Constraint::Length(2), // Hand type
                Constraint::Length(3), // Chips x Mult display
                Constraint::Length(1), // Separator
                Constraint::Length(3), // Game info (hands + discards)
                Constraint::Length(2), // Money
                Constraint::Min(0),    // Spacer
                Constraint::Length(2), // Ante / Round meta
            ])
            .split(inner);

            self.render_blind_banner(sections[0], buf);
            self.render_blind_info(sections[1], buf);
            self.render_round_score(sections[2], buf);
            self.render_hand_type(sections[3], buf);
            self.render_chips_mult(sections[4], buf);
            self.render_separator(sections[5], buf);
            self.render_game_info(sections[6], buf);
            self.render_money(sections[7], buf);
            self.render_meta(sections[9], buf);
        }
    }
}

impl SidebarWidget {
    fn render_blind_banner(&self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        // Fill background with blind color (dimmed)
        let bg_style = Style::default()
            .fg(self.blind_color)
            .add_modifier(Modifier::BOLD);

        // Center the blind name
        let name = &self.blind_name;
        let padded = Rect::new(
            area.x + 1,
            area.y,
            area.width.saturating_sub(2),
            area.height,
        );

        // Top decoration line
        if padded.height >= 1 {
            let deco: String = "\u{2500}".repeat(padded.width as usize);
            buf.set_string(
                padded.x,
                padded.y,
                &deco,
                Style::default().fg(self.blind_color),
            );
        }

        // Blind name (centered)
        if padded.height >= 2 {
            let x = padded.x + padded.width.saturating_sub(name.len() as u16) / 2;
            buf.set_string(x, padded.y + 1, name, bg_style);
        }

        // Bottom decoration line
        if padded.height >= 3 {
            let deco: String = "\u{2500}".repeat(padded.width as usize);
            buf.set_string(
                padded.x,
                padded.y + 2,
                &deco,
                Style::default().fg(self.blind_color),
            );
        }
    }

    fn render_blind_info(&self, area: Rect, buf: &mut Buffer) {
        let padded = Rect::new(
            area.x + 1,
            area.y,
            area.width.saturating_sub(2),
            area.height,
        );
        if padded.height < 2 {
            return;
        }

        // Target score
        let target_line = Line::from(vec![
            Span::styled(" Target: ", Style::default().fg(Theme::MUTED_TEXT)),
            Span::styled(
                format_number(self.score_target),
                Style::default()
                    .fg(Theme::CHIPS_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        buf.set_line(padded.x, padded.y + 1, &target_line, padded.width);

        // Reward
        if padded.height >= 3 {
            let dollars: String = "$".repeat(self.reward as usize);
            let reward_line = Line::from(vec![
                Span::styled(" Reward: ", Style::default().fg(Theme::MUTED_TEXT)),
                Span::styled(
                    dollars,
                    Style::default()
                        .fg(Theme::MONEY_COLOR)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            buf.set_line(padded.x, padded.y + 2, &reward_line, padded.width);
        }
    }

    fn render_round_score(&self, area: Rect, buf: &mut Buffer) {
        let padded = Rect::new(
            area.x + 1,
            area.y,
            area.width.saturating_sub(2),
            area.height,
        );
        if padded.height < 2 {
            return;
        }

        let label = Line::from(Span::styled(
            " Round Score",
            Style::default().fg(Theme::MUTED_TEXT),
        ));
        buf.set_line(padded.x, padded.y, &label, padded.width);

        let score_line = Line::from(Span::styled(
            format!(" {}", format_number(self.round_score)),
            Style::default()
                .fg(Theme::SCORE_COLOR)
                .add_modifier(Modifier::BOLD),
        ));
        buf.set_line(padded.x, padded.y + 1, &score_line, padded.width);
    }

    fn render_hand_type(&self, area: Rect, buf: &mut Buffer) {
        let padded = Rect::new(
            area.x + 1,
            area.y,
            area.width.saturating_sub(2),
            area.height,
        );
        if padded.height < 1 || self.hand_name.is_empty() {
            return;
        }

        let hand_line = Line::from(vec![
            Span::styled(
                format!(" {}", self.hand_name),
                Style::default()
                    .fg(Theme::BRIGHT_TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" Lv.{}", self.hand_level),
                Style::default().fg(Theme::GOLD),
            ),
        ]);
        buf.set_line(padded.x, padded.y, &hand_line, padded.width);
    }

    fn render_chips_mult(&self, area: Rect, buf: &mut Buffer) {
        let padded = Rect::new(
            area.x + 1,
            area.y,
            area.width.saturating_sub(2),
            area.height,
        );
        if padded.height < 1 {
            return;
        }

        // Render the prominent chips x mult display
        // Format: [ chips ] x [ mult ]
        let chips_str = format!("{}", self.chips);
        let mult_str = format!("{}", self.mult);

        // Calculate positions for centered display
        // chips_box_width = chips_str.len() + 4 (2 padding + 2 border chars)
        let chips_box_w = chips_str.len() as u16 + 4;
        let mult_box_w = mult_str.len() as u16 + 4;
        let separator_w = 3u16; // " x "
        let total_w = chips_box_w + separator_w + mult_box_w;
        let start_x = padded.x + padded.width.saturating_sub(total_w) / 2;

        let y = padded.y + (padded.height.saturating_sub(1)) / 2;

        // Chips box
        let chips_box_str = format!(" {} ", chips_str);
        buf.set_string(
            start_x,
            y,
            &chips_box_str,
            Style::default()
                .fg(Theme::CHIPS_COLOR)
                .add_modifier(Modifier::BOLD),
        );

        // Separator " x "
        buf.set_string(
            start_x + chips_box_w,
            y,
            " \u{00d7} ",
            Style::default()
                .fg(Theme::BRIGHT_TEXT)
                .add_modifier(Modifier::BOLD),
        );

        // Mult box
        let mult_box_str = format!(" {} ", mult_str);
        buf.set_string(
            start_x + chips_box_w + separator_w,
            y,
            &mult_box_str,
            Style::default()
                .fg(Theme::MULT_COLOR)
                .add_modifier(Modifier::BOLD),
        );
    }

    fn render_separator(&self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }
        let line: String = "\u{2500}".repeat(area.width.saturating_sub(2) as usize);
        buf.set_string(
            area.x + 1,
            area.y,
            &line,
            Style::default().fg(Theme::DIM_TEXT),
        );
    }

    fn render_game_info(&self, area: Rect, buf: &mut Buffer) {
        let padded = Rect::new(
            area.x + 1,
            area.y,
            area.width.saturating_sub(2),
            area.height,
        );
        if padded.height < 2 {
            return;
        }

        // Hands remaining
        let hands_line = Line::from(vec![
            Span::styled(" Hands:    ", Style::default().fg(Theme::MUTED_TEXT)),
            Span::styled(
                format!("{}", self.hands_remaining),
                Style::default()
                    .fg(Theme::CHIPS_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        buf.set_line(padded.x, padded.y, &hands_line, padded.width);

        // Discards remaining
        if padded.height >= 2 {
            let disc_line = Line::from(vec![
                Span::styled(" Discards: ", Style::default().fg(Theme::MUTED_TEXT)),
                Span::styled(
                    format!("{}", self.discards_remaining),
                    Style::default()
                        .fg(Theme::MULT_COLOR)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            buf.set_line(padded.x, padded.y + 1, &disc_line, padded.width);
        }
    }

    fn render_money(&self, area: Rect, buf: &mut Buffer) {
        let padded = Rect::new(
            area.x + 1,
            area.y,
            area.width.saturating_sub(2),
            area.height,
        );
        if padded.height < 1 {
            return;
        }

        let money_line = Line::from(Span::styled(
            format!(" ${}", self.money),
            Style::default()
                .fg(Theme::MONEY_COLOR)
                .add_modifier(Modifier::BOLD),
        ));
        buf.set_line(padded.x, padded.y, &money_line, padded.width);
    }

    fn render_meta(&self, area: Rect, buf: &mut Buffer) {
        let padded = Rect::new(
            area.x + 1,
            area.y,
            area.width.saturating_sub(2),
            area.height,
        );
        if padded.height < 1 {
            return;
        }

        let meta_line = Line::from(vec![
            Span::styled(" Ante: ", Style::default().fg(Theme::MUTED_TEXT)),
            Span::styled(
                format!("{}/{}", self.ante, self.max_ante),
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Round: ", Style::default().fg(Theme::MUTED_TEXT)),
            Span::styled(
                format!("{}", self.round_number),
                Style::default()
                    .fg(Theme::BRIGHT_TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        buf.set_line(padded.x, padded.y, &meta_line, padded.width);
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
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
