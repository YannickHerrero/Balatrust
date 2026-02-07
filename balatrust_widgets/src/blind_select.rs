use balatrust_core::blind::{self, BlindType, BossBlind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Widget};

use crate::theme::Theme;

/// Blind selection screen widget
pub struct BlindSelectWidget {
    pub ante: u8,
    pub boss: BossBlind,
    pub cursor: usize, // 0=small, 1=big, 2=boss
    pub money: u32,
}

impl BlindSelectWidget {
    pub fn new(ante: u8, boss: BossBlind, cursor: usize, money: u32) -> Self {
        Self {
            ante,
            boss,
            cursor,
            money,
        }
    }
}

impl Widget for BlindSelectWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Title
        let title = Line::from(vec![
            Span::styled(
                format!("  Ante {}  ", self.ante),
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "Choose Your Blind",
                Style::default()
                    .fg(Theme::BRIGHT_TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("${}  ", self.money),
                Style::default()
                    .fg(Theme::MONEY_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let outer = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Theme::CARD_BORDER))
            .title(title)
            .title_alignment(Alignment::Center);

        let inner = outer.inner(area);
        outer.render(area, buf);

        if inner.width < 40 || inner.height < 10 {
            return;
        }

        // Three columns for blinds
        let columns = Layout::horizontal([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(inner);

        let blinds = [BlindType::Small, BlindType::Big, BlindType::Boss(self.boss)];

        for (i, (blind, col)) in blinds.iter().zip(columns.iter()).enumerate() {
            let is_selected = self.cursor == i;
            render_blind_card(*blind, *col, buf, is_selected, self.ante);
        }
    }
}

fn render_blind_card(blind: BlindType, area: Rect, buf: &mut Buffer, selected: bool, ante: u8) {
    let (color, name) = match &blind {
        BlindType::Small => (Theme::SMALL_BLIND, "SMALL BLIND"),
        BlindType::Big => (Theme::BIG_BLIND, "BIG BLIND"),
        BlindType::Boss(_) => (Theme::BOSS_BLIND, "BOSS BLIND"),
    };

    // Pad the card area
    let padded = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    let border_style = if selected {
        Style::default()
            .fg(Theme::CARD_SELECTED)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    };

    let border_type = if selected {
        BorderType::Double
    } else {
        BorderType::Rounded
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(border_style)
        .padding(Padding::uniform(1));

    let card_inner = block.inner(padded);
    block.render(padded, buf);

    if card_inner.height < 6 {
        return;
    }

    let mut y = card_inner.y;

    // Blind name
    let name_style = Style::default().fg(color).add_modifier(Modifier::BOLD);

    let centered_name = center_text(name, card_inner.width as usize);
    buf.set_string(card_inner.x, y, &centered_name, name_style);
    y += 2;

    // Score target
    let target = blind::score_target(ante, &blind);
    let target_str = format!("{}", target);
    let target_centered = center_text(&target_str, card_inner.width as usize);
    buf.set_string(
        card_inner.x,
        y,
        &target_centered,
        Style::default()
            .fg(Theme::BRIGHT_TEXT)
            .add_modifier(Modifier::BOLD),
    );
    y += 1;

    let chips_label = center_text("chips", card_inner.width as usize);
    buf.set_string(
        card_inner.x,
        y,
        &chips_label,
        Style::default().fg(Theme::MUTED_TEXT),
    );
    y += 2;

    // Reward
    let reward = blind.reward();
    let reward_str = format!("${}", reward);
    let reward_centered = center_text(&reward_str, card_inner.width as usize);
    buf.set_string(
        card_inner.x,
        y,
        &reward_centered,
        Style::default().fg(Theme::MONEY_COLOR),
    );
    y += 2;

    // Boss description
    if let BlindType::Boss(boss) = &blind {
        if y < card_inner.bottom() {
            let desc = boss.description();
            // Word wrap
            for line in word_wrap(desc, card_inner.width as usize) {
                if y >= card_inner.bottom() {
                    break;
                }
                buf.set_string(
                    card_inner.x,
                    y,
                    &line,
                    Style::default().fg(Theme::MULT_COLOR),
                );
                y += 1;
            }
        }
    }

    // Skip / Play hint
    if y + 1 < card_inner.bottom() {
        y = card_inner.bottom() - 1;
        if selected {
            let hint = if blind.can_skip() {
                "[Enter] Play  [S] Skip"
            } else {
                "[Enter] Play"
            };
            let hint_centered = center_text(hint, card_inner.width as usize);
            buf.set_string(
                card_inner.x,
                y,
                &hint_centered,
                Style::default().fg(Theme::GOLD),
            );
        }
    }
}

fn center_text(text: &str, width: usize) -> String {
    let text_len = text.len();
    if text_len >= width {
        return text[..width].to_string();
    }
    let padding = (width - text_len) / 2;
    format!("{:>width$}", text, width = padding + text_len)
}

fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
