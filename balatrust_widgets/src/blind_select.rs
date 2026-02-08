use balatrust_core::blind::{self, BlindType, BossBlind};
use balatrust_core::run::BlindOutcome;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;

use crate::theme::Theme;

/// Blind selection panel widget (renders in the center area).
///
/// Displays 3 blind cards side by side with their current state.
pub struct BlindSelectWidget {
    pub ante: u8,
    pub boss: BossBlind,
    pub cursor: usize, // 0=small, 1=big, 2=boss
    pub outcomes: [BlindOutcome; 3],
}

impl BlindSelectWidget {
    pub fn new(ante: u8, boss: BossBlind, cursor: usize, outcomes: [BlindOutcome; 3]) -> Self {
        Self {
            ante,
            boss,
            cursor,
            outcomes,
        }
    }

    /// Get the Rect for a specific blind card given the panel area.
    pub fn card_rect(area: Rect, index: usize) -> Rect {
        if area.width < 30 || area.height < 8 {
            return Rect::default();
        }

        let columns = Layout::horizontal([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(area);

        if index >= 3 {
            return Rect::default();
        }

        let col = columns[index];
        // Pad inside each column
        Rect {
            x: col.x + 1,
            y: col.y,
            width: col.width.saturating_sub(2),
            height: col.height,
        }
    }

    /// Get the Rect for the "Select" button on a card (top header area).
    pub fn select_button_rect(area: Rect, index: usize) -> Rect {
        let card = Self::card_rect(area, index);
        if card.width < 6 || card.height < 4 {
            return Rect::default();
        }
        // The select button is the top 3 rows of the card inner area
        // (inside the card border: 1 row top border + 1 padding)
        let inner_x = card.x + 2;
        let inner_y = card.y + 1;
        let inner_w = card.width.saturating_sub(4);
        Rect::new(inner_x, inner_y, inner_w, 3)
    }

    /// Get the Rect for the "Skip" button on a card.
    /// Positioned below the "Select" button.
    pub fn skip_button_rect(area: Rect, index: usize) -> Rect {
        let card = Self::card_rect(area, index);
        if card.width < 6 || card.height < 8 {
            return Rect::default();
        }
        let inner_x = card.x + 2;
        let inner_y = card.y + 4; // Below the select button (3 rows) + 1 gap
        let inner_w = card.width.saturating_sub(4);
        Rect::new(inner_x, inner_y, inner_w, 3)
    }
}

impl Widget for BlindSelectWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 30 || area.height < 8 {
            return;
        }

        let columns = Layout::horizontal([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(area);

        let blinds = [BlindType::Small, BlindType::Big, BlindType::Boss(self.boss)];

        for (i, (blind, col)) in blinds.iter().zip(columns.iter()).enumerate() {
            let padded = Rect {
                x: col.x + 1,
                y: col.y,
                width: col.width.saturating_sub(2),
                height: col.height,
            };
            let is_cursor = self.cursor == i;
            render_blind_card(*blind, padded, buf, is_cursor, self.ante, self.outcomes[i]);
        }
    }
}

fn render_blind_card(
    blind: BlindType,
    area: Rect,
    buf: &mut Buffer,
    is_cursor: bool,
    ante: u8,
    outcome: BlindOutcome,
) {
    if area.width < 10 || area.height < 6 {
        return;
    }

    let (base_color, blind_name) = match &blind {
        BlindType::Small => (Theme::SMALL_BLIND, "Small Blind"),
        BlindType::Big => (Theme::BIG_BLIND, "Big Blind"),
        BlindType::Boss(boss) => (Theme::BOSS_BLIND, boss_name(boss)),
    };

    let is_active = outcome == BlindOutcome::Active;
    let is_dimmed = matches!(outcome, BlindOutcome::Skipped | BlindOutcome::Beaten);
    let is_upcoming = outcome == BlindOutcome::Upcoming;

    // Choose border style based on state
    let border_color = if is_cursor && is_active {
        Theme::CARD_SELECTED
    } else if is_active {
        base_color
    } else {
        Theme::DIM_TEXT
    };

    let border_style = Style::default().fg(border_color);

    // Draw the card border (rounded or double for cursor)
    if is_cursor && is_active {
        draw_double_border(area, buf, border_style);
    } else {
        draw_rounded_border(area, buf, border_style);
    }

    // Inner content area (inside border + 1 col padding each side)
    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    if inner.width < 6 || inner.height < 4 {
        return;
    }

    let mut y = inner.y;
    let max_w = inner.width as usize;

    // ═══ 1. Status Header / Buttons ═══
    match outcome {
        BlindOutcome::Active => {
            // "Select" button (orange)
            if inner.height >= 3 {
                render_button(
                    Rect::new(inner.x, y, inner.width, 3),
                    buf,
                    "Select",
                    Theme::GOLD,
                );
                y += 3;

                // "Skip" button for Small/Big (dimmed)
                if blind.can_skip() && y + 3 <= inner.bottom() {
                    render_button(
                        Rect::new(inner.x, y, inner.width, 3),
                        buf,
                        "Skip",
                        Theme::MUTED_TEXT,
                    );
                    y += 3;
                }
            }
        }
        BlindOutcome::Skipped => {
            let label = "SKIPPED";
            let x = inner.x + (max_w as u16).saturating_sub(label.len() as u16) / 2;
            buf.set_string(
                x,
                y,
                label,
                Style::default()
                    .fg(Theme::DIM_TEXT)
                    .add_modifier(Modifier::BOLD),
            );
            y += 2;
        }
        BlindOutcome::Beaten => {
            let label = "BEATEN";
            let x = inner.x + (max_w as u16).saturating_sub(label.len() as u16) / 2;
            buf.set_string(
                x,
                y,
                label,
                Style::default()
                    .fg(Theme::MONEY_COLOR)
                    .add_modifier(Modifier::BOLD),
            );
            y += 2;
        }
        BlindOutcome::Upcoming => {
            let label = "UPCOMING";
            let x = inner.x + (max_w as u16).saturating_sub(label.len() as u16) / 2;
            buf.set_string(
                x,
                y,
                label,
                Style::default()
                    .fg(Theme::DIM_TEXT)
                    .add_modifier(Modifier::BOLD),
            );
            y += 2;
        }
    }

    // ═══ Separator ═══
    if y < inner.bottom() {
        let sep: String = "\u{2500}".repeat(max_w);
        let sep_color = if is_dimmed || is_upcoming {
            Theme::DIM_TEXT
        } else {
            base_color
        };
        buf.set_string(inner.x, y, &sep, Style::default().fg(sep_color));
        y += 1;
    }

    // ═══ 2. Blind Name ═══
    if y < inner.bottom() {
        let name_style = if is_dimmed || is_upcoming {
            Style::default()
                .fg(Theme::DIM_TEXT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(base_color).add_modifier(Modifier::BOLD)
        };
        let display_name: String = blind_name.chars().take(max_w).collect();
        let x = inner.x + (max_w as u16).saturating_sub(display_name.len() as u16) / 2;
        buf.set_string(x, y, &display_name, name_style);
        y += 1;
    }

    // ═══ 3. Blind Icon (simplified TUI representation) ═══
    if y + 1 < inner.bottom() {
        y += 1; // blank line
        let icon = match &blind {
            BlindType::Small => "\u{25cf}",   // ●
            BlindType::Big => "\u{25c6}",     // ◆
            BlindType::Boss(_) => "\u{2605}", // ★
        };
        let icon_style = if is_dimmed || is_upcoming {
            Style::default().fg(Theme::DIM_TEXT)
        } else {
            Style::default().fg(base_color).add_modifier(Modifier::BOLD)
        };
        let x = inner.x + inner.width / 2;
        buf.set_string(x, y, icon, icon_style);
        y += 2; // icon + blank line
    }

    // ═══ 4. Boss Effect Description (boss only) ═══
    if let BlindType::Boss(boss) = &blind {
        let desc = boss.description();
        let lines = word_wrap(desc, max_w);
        let desc_style = if is_dimmed || is_upcoming {
            Style::default().fg(Theme::DIM_TEXT)
        } else {
            Style::default().fg(Theme::MULT_COLOR)
        };
        for line in &lines {
            if y >= inner.bottom() {
                break;
            }
            let x = inner.x + (max_w as u16).saturating_sub(line.len() as u16) / 2;
            buf.set_string(x, y, line, desc_style);
            y += 1;
        }
        if y < inner.bottom() {
            y += 1; // blank line after description
        }
    }

    // ═══ 5. Score Requirement ═══
    if y < inner.bottom() {
        let label = "Score at least";
        let label_style = if is_dimmed || is_upcoming {
            Style::default().fg(Theme::DIM_TEXT)
        } else {
            Style::default().fg(Theme::MUTED_TEXT)
        };
        let x = inner.x + (max_w as u16).saturating_sub(label.len() as u16) / 2;
        buf.set_string(x, y, label, label_style);
        y += 1;
    }

    if y < inner.bottom() {
        let target = blind::score_target(ante, &blind);
        let target_str = format_number(target);
        let target_style = if is_dimmed || is_upcoming {
            Style::default()
                .fg(Theme::DIM_TEXT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Theme::CHIPS_COLOR)
                .add_modifier(Modifier::BOLD)
        };
        let x = inner.x + (max_w as u16).saturating_sub(target_str.len() as u16) / 2;
        buf.set_string(x, y, &target_str, target_style);
        y += 2;
    }

    // ═══ 6. Reward ═══
    if y < inner.bottom() {
        let reward = blind.reward();
        let dollars: String = "$".repeat(reward as usize);
        let reward_style = if is_dimmed || is_upcoming {
            Style::default().fg(Theme::DIM_TEXT)
        } else {
            Style::default()
                .fg(Theme::MONEY_COLOR)
                .add_modifier(Modifier::BOLD)
        };
        let label = format!("Reward: {}", dollars);
        let x = inner.x + (max_w as u16).saturating_sub(label.len() as u16) / 2;
        buf.set_string(x, y, &label, reward_style);
        y += 2;
    }

    // ═══ 7. Ante Progression Info (boss only) ═══
    if let BlindType::Boss(_) = &blind {
        if is_active && y + 2 < inner.bottom() {
            let ante_lines = ["Reach the ante", "Raise all Blinds", "Refresh the Blinds"];
            for line in &ante_lines {
                if y >= inner.bottom() {
                    break;
                }
                let x = inner.x + (max_w as u16).saturating_sub(line.len() as u16) / 2;
                buf.set_string(x, y, line, Style::default().fg(Theme::GOLD));
                y += 1;
            }
        }
    }

    // ═══ SKIPPED overlay (large stamp for skipped cards) ═══
    if outcome == BlindOutcome::Skipped {
        // Render a large "SKIPPED" stamp across the center of the card
        let stamp = "SKIPPED";
        let stamp_y = area.y + area.height / 2;
        if stamp_y < area.bottom() {
            let x = area.x + area.width.saturating_sub(stamp.len() as u16) / 2;
            buf.set_string(
                x,
                stamp_y,
                stamp,
                Style::default()
                    .fg(Theme::DIM_TEXT)
                    .add_modifier(Modifier::BOLD),
            );
        }
    }
}

fn render_button(area: Rect, buf: &mut Buffer, label: &str, color: ratatui::style::Color) {
    if area.width < 5 || area.height < 3 {
        return;
    }

    let border_style = Style::default().fg(color);
    let text_style = Style::default().fg(color).add_modifier(Modifier::BOLD);

    // Top: ╭──╮
    let mut top = String::new();
    top.push('\u{256d}');
    for _ in 1..area.width.saturating_sub(1) {
        top.push('\u{2500}');
    }
    top.push('\u{256e}');
    buf.set_string(area.x, area.y, &top, border_style);

    // Middle: │ label │
    let y = area.y + 1;
    buf.set_string(area.x, y, "\u{2502}", border_style);
    let inner_w = area.width.saturating_sub(2) as usize;
    let padding: String = " ".repeat(inner_w);
    buf.set_string(area.x + 1, y, &padding, Style::default());
    let display_label: String = label.chars().take(inner_w).collect();
    let label_x = area.x + 1 + (inner_w as u16).saturating_sub(display_label.len() as u16) / 2;
    buf.set_string(label_x, y, &display_label, text_style);
    buf.set_string(area.x + area.width - 1, y, "\u{2502}", border_style);

    // Bottom: ╰──╯
    let y = area.y + 2;
    let mut bot = String::new();
    bot.push('\u{2570}');
    for _ in 1..area.width.saturating_sub(1) {
        bot.push('\u{2500}');
    }
    bot.push('\u{256f}');
    buf.set_string(area.x, y, &bot, border_style);
}

fn draw_rounded_border(area: Rect, buf: &mut Buffer, style: Style) {
    // Top
    buf.set_string(area.x, area.y, "\u{256d}", style);
    for x in 1..area.width.saturating_sub(1) {
        buf.set_string(area.x + x, area.y, "\u{2500}", style);
    }
    buf.set_string(area.x + area.width - 1, area.y, "\u{256e}", style);

    // Sides
    for y in 1..area.height.saturating_sub(1) {
        buf.set_string(area.x, area.y + y, "\u{2502}", style);
        buf.set_string(area.x + area.width - 1, area.y + y, "\u{2502}", style);
        // Clear interior
        for x in 1..area.width.saturating_sub(1) {
            buf.set_string(area.x + x, area.y + y, " ", Style::default());
        }
    }

    // Bottom
    let by = area.y + area.height - 1;
    buf.set_string(area.x, by, "\u{2570}", style);
    for x in 1..area.width.saturating_sub(1) {
        buf.set_string(area.x + x, by, "\u{2500}", style);
    }
    buf.set_string(area.x + area.width - 1, by, "\u{256f}", style);
}

fn draw_double_border(area: Rect, buf: &mut Buffer, style: Style) {
    // Top
    buf.set_string(area.x, area.y, "\u{2554}", style); // ╔
    for x in 1..area.width.saturating_sub(1) {
        buf.set_string(area.x + x, area.y, "\u{2550}", style); // ═
    }
    buf.set_string(area.x + area.width - 1, area.y, "\u{2557}", style); // ╗

    // Sides
    for y in 1..area.height.saturating_sub(1) {
        buf.set_string(area.x, area.y + y, "\u{2551}", style); // ║
        buf.set_string(area.x + area.width - 1, area.y + y, "\u{2551}", style); // ║
        for x in 1..area.width.saturating_sub(1) {
            buf.set_string(area.x + x, area.y + y, " ", Style::default());
        }
    }

    // Bottom
    let by = area.y + area.height - 1;
    buf.set_string(area.x, by, "\u{255a}", style); // ╚
    for x in 1..area.width.saturating_sub(1) {
        buf.set_string(area.x + x, by, "\u{2550}", style); // ═
    }
    buf.set_string(area.x + area.width - 1, by, "\u{255d}", style); // ╝
}

fn boss_name(boss: &BossBlind) -> &'static str {
    match boss {
        BossBlind::TheHook => "The Hook",
        BossBlind::TheWall => "The Wall",
        BossBlind::ThePsychic => "The Psychic",
        BossBlind::TheNeedle => "The Needle",
        BossBlind::TheClub => "The Club",
        BossBlind::TheGoad => "The Goad",
        BossBlind::TheWindow => "The Window",
        BossBlind::TheHead => "The Head",
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
