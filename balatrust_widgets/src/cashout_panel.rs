use balatrust_core::run::RewardBreakdown;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Widget};

use crate::theme::Theme;

/// Cash-out panel widget displayed after beating a blind.
/// Shows a prominent "Cash Out" button, blind reward line, and earnings breakdown.
pub struct CashOutPanel {
    pub blind_name: String,
    pub score_target: u64,
    pub breakdown: RewardBreakdown,
}

impl CashOutPanel {
    pub fn new(blind_name: String, score_target: u64, breakdown: RewardBreakdown) -> Self {
        Self {
            blind_name,
            score_target,
            breakdown,
        }
    }

    /// Calculate the rect for the "Cash Out" button given the widget area.
    /// The button is in the top portion of the inner panel.
    pub fn cashout_button_rect(area: Rect) -> Rect {
        let panel = Self::panel_rect(area);
        let inner = Self::inner_rect(panel);
        if inner.height < 5 || inner.width < 10 {
            return Rect::default();
        }

        // Button is in the first row of the inner layout
        let rows = Layout::vertical([
            Constraint::Length(3), // Cash out button
            Constraint::Min(0),    // Rest
        ])
        .split(inner);

        // Add horizontal padding to the button
        let btn_padding = 2u16;
        Rect::new(
            rows[0].x + btn_padding,
            rows[0].y,
            rows[0].width.saturating_sub(btn_padding * 2),
            rows[0].height,
        )
    }

    /// Check if a point hits the cash-out button
    pub fn hit_test_cashout(area: Rect, col: u16, row: u16) -> bool {
        let btn = Self::cashout_button_rect(area);
        btn.width > 0
            && col >= btn.x
            && col < btn.x + btn.width
            && row >= btn.y
            && row < btn.y + btn.height
    }

    fn panel_rect(area: Rect) -> Rect {
        // The panel fills most of the center area with some margin
        let h_margin = 2u16;
        let v_margin = 1u16;
        Rect::new(
            area.x + h_margin,
            area.y + v_margin,
            area.width.saturating_sub(h_margin * 2),
            area.height.saturating_sub(v_margin * 2),
        )
    }

    fn inner_rect(panel: Rect) -> Rect {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .padding(Padding::new(2, 2, 1, 1));
        block.inner(panel)
    }
}

impl Widget for CashOutPanel {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 10 {
            return;
        }

        let panel = Self::panel_rect(area);

        // Draw the outer panel border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Theme::CARD_SELECTED))
            .padding(Padding::new(2, 2, 1, 1));

        let inner = block.inner(panel);
        block.render(panel, buf);

        if inner.height < 5 || inner.width < 10 {
            return;
        }

        // Inner vertical layout
        let rows = Layout::vertical([
            Constraint::Length(3), // Cash out button
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Blind reward line
            Constraint::Length(1), // Dotted separator
            Constraint::Min(0),    // Earnings breakdown
        ])
        .split(inner);

        // === 1. Cash Out Button ===
        self.render_cashout_button(rows[0], buf);

        // === 2. Blind Reward Line ===
        self.render_blind_reward_line(rows[2], buf);

        // === 3. Dotted Separator ===
        self.render_dotted_separator(rows[3], buf);

        // === 4. Earnings Breakdown ===
        self.render_earnings_breakdown(rows[4], buf);
    }
}

impl CashOutPanel {
    fn render_cashout_button(&self, area: Rect, buf: &mut Buffer) {
        let btn_padding = 2u16;
        let btn_area = Rect::new(
            area.x + btn_padding,
            area.y,
            area.width.saturating_sub(btn_padding * 2),
            area.height,
        );

        if btn_area.width < 10 || btn_area.height < 3 {
            return;
        }

        let border_style = Style::default().fg(Theme::GOLD);
        let text_style = Style::default()
            .fg(Theme::GOLD)
            .add_modifier(Modifier::BOLD);

        // Top border
        let mut top = String::new();
        top.push('\u{256d}'); // ╭
        for _ in 1..btn_area.width.saturating_sub(1) {
            top.push('\u{2500}'); // ─
        }
        top.push('\u{256e}'); // ╮
        buf.set_string(btn_area.x, btn_area.y, &top, border_style);

        // Middle row (content)
        let y = btn_area.y + 1;
        buf.set_string(btn_area.x, y, "\u{2502}", border_style);
        let inner_w = btn_area.width.saturating_sub(2) as usize;
        let padding: String = " ".repeat(inner_w);
        buf.set_string(btn_area.x + 1, y, &padding, Style::default());

        let label = format!("Cash Out: ${}", self.breakdown.total);
        let label_display: String = label.chars().take(inner_w).collect();
        let label_x =
            btn_area.x + 1 + (inner_w as u16).saturating_sub(label_display.len() as u16) / 2;
        buf.set_string(label_x, y, &label_display, text_style);
        buf.set_string(btn_area.x + btn_area.width - 1, y, "\u{2502}", border_style);

        // Bottom border
        let y = btn_area.y + 2;
        let mut bot = String::new();
        bot.push('\u{2570}'); // ╰
        for _ in 1..btn_area.width.saturating_sub(1) {
            bot.push('\u{2500}'); // ─
        }
        bot.push('\u{256f}'); // ╯
        buf.set_string(btn_area.x, y, &bot, border_style);
    }

    fn render_blind_reward_line(&self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 {
            return;
        }

        let dollars: String = "$".repeat(self.breakdown.blind_reward as usize);
        let line = Line::from(vec![
            Span::styled(
                format!(" {} ", self.blind_name),
                Style::default()
                    .fg(Theme::BRIGHT_TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("\u{2265}{}", format_number(self.score_target)),
                Style::default()
                    .fg(Theme::MULT_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            // Right-align the dollar signs
            Span::styled(
                format!(
                    "{:>width$}",
                    dollars,
                    width = (area.width as usize)
                        .saturating_sub(self.blind_name.len() + 2)
                        .saturating_sub(format_number(self.score_target).len() + 1)
                        .saturating_sub(dollars.len())
                        + dollars.len()
                ),
                Style::default()
                    .fg(Theme::MULT_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        buf.set_line(area.x, area.y, &line, area.width);
    }

    fn render_dotted_separator(&self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 {
            return;
        }
        let dots: String = "\u{00b7} "
            .repeat(area.width as usize / 2)
            .chars()
            .take(area.width as usize)
            .collect();
        buf.set_string(area.x, area.y, &dots, Style::default().fg(Theme::DIM_TEXT));
    }

    fn render_earnings_breakdown(&self, area: Rect, buf: &mut Buffer) {
        if area.width < 15 {
            return;
        }

        let mut y = area.y;

        // Each earning line: amount (blue) | description (white) | dollar signs (red)
        let lines = self.build_earnings_lines();

        for (amount, description, dollar_count) in &lines {
            if y >= area.bottom() {
                break;
            }

            let dollars: String = "$".repeat(*dollar_count as usize);
            let amount_str = format!("{}", amount);

            // Left: amount in blue
            buf.set_string(
                area.x + 1,
                y,
                &amount_str,
                Style::default()
                    .fg(Theme::CHIPS_COLOR)
                    .add_modifier(Modifier::BOLD),
            );

            // Center: description
            let desc_x = area.x + 1 + amount_str.len() as u16 + 1;
            let desc_max = area
                .width
                .saturating_sub(amount_str.len() as u16 + 2 + dollars.len() as u16 + 2)
                as usize;
            let desc_display: String = description.chars().take(desc_max).collect();
            buf.set_string(
                desc_x,
                y,
                &desc_display,
                Style::default().fg(Theme::MUTED_TEXT),
            );

            // Right: dollar signs in red
            let dollars_x = area.x + area.width - dollars.len() as u16 - 1;
            buf.set_string(
                dollars_x,
                y,
                &dollars,
                Style::default()
                    .fg(Theme::MULT_COLOR)
                    .add_modifier(Modifier::BOLD),
            );

            y += 1;
        }
    }

    fn build_earnings_lines(&self) -> Vec<(u32, String, u32)> {
        let mut lines = Vec::new();
        let bd = &self.breakdown;

        // Remaining hands bonus
        if bd.hands_bonus > 0 {
            lines.push((
                bd.hands_bonus,
                format!("Remaining hands ($1 each)",),
                bd.hands_bonus,
            ));
        }

        // Interest
        if bd.interest > 0 {
            lines.push((
                bd.interest,
                format!("Interest: $1 per $5 held (max 5)"),
                bd.interest,
            ));
        }

        // Golden Joker bonus
        if bd.golden_joker_bonus > 0 {
            lines.push((
                bd.golden_joker_bonus,
                "Golden Joker".to_string(),
                bd.golden_joker_bonus,
            ));
        }

        lines
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
