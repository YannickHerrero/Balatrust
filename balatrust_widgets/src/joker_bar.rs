use balatrust_core::joker::{Joker, JokerRarity};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;

use crate::theme::Theme;

pub const JOKER_WIDTH: u16 = 13;
pub const JOKER_HEIGHT: u16 = 5;

/// Widget showing the joker bar (horizontal row of owned jokers)
pub struct JokerBarWidget<'a> {
    pub jokers: &'a [Joker],
    pub max_jokers: u8,
    pub selected: Option<usize>,
    /// Index of the joker currently "activated" (pulsing during scoring animation)
    pub activated: Option<usize>,
}

impl<'a> JokerBarWidget<'a> {
    pub fn new(jokers: &'a [Joker], max_jokers: u8) -> Self {
        Self {
            jokers,
            max_jokers,
            selected: None,
            activated: None,
        }
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn activated(mut self, activated: Option<usize>) -> Self {
        self.activated = activated;
        self
    }

    /// Get the Rect for a specific joker given the bar area
    pub fn joker_rect(&self, area: Rect, joker_index: usize) -> Option<Rect> {
        if joker_index >= self.max_jokers as usize {
            return None;
        }

        let spacing = 1u16;
        let total_slots = self.max_jokers as u16;
        let total_width = total_slots * JOKER_WIDTH + (total_slots.saturating_sub(1)) * spacing;
        let start_x = area.x + area.width.saturating_sub(total_width) / 2;

        let x = start_x + (joker_index as u16) * (JOKER_WIDTH + spacing);
        Some(Rect::new(x, area.y, JOKER_WIDTH, JOKER_HEIGHT))
    }
}

impl<'a> Widget for JokerBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < JOKER_HEIGHT {
            return;
        }

        let spacing = 1u16;
        let total_slots = self.max_jokers as u16;
        let total_width = total_slots * JOKER_WIDTH + (total_slots.saturating_sub(1)) * spacing;
        let start_x = area.x + area.width.saturating_sub(total_width) / 2;

        for i in 0..self.max_jokers as usize {
            let x = start_x + (i as u16) * (JOKER_WIDTH + spacing);
            let card_area = Rect::new(x, area.y, JOKER_WIDTH, JOKER_HEIGHT);

            if card_area.right() > area.right() {
                break;
            }

            if let Some(joker) = self.jokers.get(i) {
                let is_selected = self.selected == Some(i);
                let is_activated = self.activated == Some(i);
                render_joker_card(joker, card_area, buf, is_selected, is_activated);
            } else {
                render_empty_slot(card_area, buf);
            }
        }
    }
}

fn render_joker_card(joker: &Joker, area: Rect, buf: &mut Buffer, selected: bool, activated: bool) {
    let rarity_color = match joker.joker_type.rarity() {
        JokerRarity::Common => Theme::COMMON,
        JokerRarity::Uncommon => Theme::UNCOMMON,
        JokerRarity::Rare => Theme::RARE,
        JokerRarity::Legendary => Theme::LEGENDARY,
    };

    let border_color = if activated {
        Theme::BRIGHT_TEXT // Bright white glow when activated during scoring
    } else if selected {
        Theme::CARD_SELECTED
    } else {
        rarity_color
    };
    let border_style = Style::default().fg(border_color);

    // Draw border
    if selected {
        buf.set_string(area.x, area.y, "\u{2554}", border_style); // ╔
        for x in 1..area.width - 1 {
            buf.set_string(area.x + x, area.y, "\u{2550}", border_style);
        }
        buf.set_string(area.x + area.width - 1, area.y, "\u{2557}", border_style);

        for y in 1..area.height - 1 {
            buf.set_string(area.x, area.y + y, "\u{2551}", border_style);
            buf.set_string(
                area.x + area.width - 1,
                area.y + y,
                "\u{2551}",
                border_style,
            );
            for x in 1..area.width - 1 {
                buf.set_string(area.x + x, area.y + y, " ", Style::default());
            }
        }

        buf.set_string(area.x, area.y + area.height - 1, "\u{255a}", border_style);
        for x in 1..area.width - 1 {
            buf.set_string(
                area.x + x,
                area.y + area.height - 1,
                "\u{2550}",
                border_style,
            );
        }
        buf.set_string(
            area.x + area.width - 1,
            area.y + area.height - 1,
            "\u{255d}",
            border_style,
        );
    } else {
        buf.set_string(area.x, area.y, "\u{256d}", border_style);
        for x in 1..area.width - 1 {
            buf.set_string(area.x + x, area.y, "\u{2500}", border_style);
        }
        buf.set_string(area.x + area.width - 1, area.y, "\u{256e}", border_style);

        for y in 1..area.height - 1 {
            buf.set_string(area.x, area.y + y, "\u{2502}", border_style);
            buf.set_string(
                area.x + area.width - 1,
                area.y + y,
                "\u{2502}",
                border_style,
            );
            for x in 1..area.width - 1 {
                buf.set_string(area.x + x, area.y + y, " ", Style::default());
            }
        }

        buf.set_string(area.x, area.y + area.height - 1, "\u{2570}", border_style);
        for x in 1..area.width - 1 {
            buf.set_string(
                area.x + x,
                area.y + area.height - 1,
                "\u{2500}",
                border_style,
            );
        }
        buf.set_string(
            area.x + area.width - 1,
            area.y + area.height - 1,
            "\u{256f}",
            border_style,
        );
    }

    // Joker name (truncated to fit)
    let name = joker.joker_type.name();
    let max_len = (area.width - 2) as usize;
    let display_name: String = name.chars().take(max_len).collect();
    buf.set_string(
        area.x + 1,
        area.y + 1,
        &display_name,
        Style::default()
            .fg(Theme::BRIGHT_TEXT)
            .add_modifier(Modifier::BOLD),
    );

    // Description (truncated)
    let desc = joker.joker_type.description();
    let display_desc: String = desc.chars().take(max_len).collect();
    buf.set_string(
        area.x + 1,
        area.y + 2,
        &display_desc,
        Style::default().fg(rarity_color),
    );

    // Rarity label
    let rarity_str = match joker.joker_type.rarity() {
        JokerRarity::Common => "Common",
        JokerRarity::Uncommon => "Uncommon",
        JokerRarity::Rare => "Rare",
        JokerRarity::Legendary => "Legend",
    };
    let rarity_display: String = rarity_str.chars().take(max_len).collect();
    buf.set_string(
        area.x + 1,
        area.y + 3,
        &rarity_display,
        Style::default().fg(Theme::DIM_TEXT),
    );
}

fn render_empty_slot(area: Rect, buf: &mut Buffer) {
    let style = Style::default().fg(Theme::DIM_TEXT);

    buf.set_string(area.x, area.y, "\u{256d}", style);
    for x in 1..area.width - 1 {
        buf.set_string(area.x + x, area.y, "\u{2500}", style);
    }
    buf.set_string(area.x + area.width - 1, area.y, "\u{256e}", style);

    for y in 1..area.height - 1 {
        buf.set_string(area.x, area.y + y, "\u{2502}", style);
        buf.set_string(area.x + area.width - 1, area.y + y, "\u{2502}", style);
        for x in 1..area.width - 1 {
            buf.set_string(area.x + x, area.y + y, " ", Style::default());
        }
    }

    buf.set_string(area.x, area.y + area.height - 1, "\u{2570}", style);
    for x in 1..area.width - 1 {
        buf.set_string(area.x + x, area.y + area.height - 1, "\u{2500}", style);
    }
    buf.set_string(
        area.x + area.width - 1,
        area.y + area.height - 1,
        "\u{256f}",
        style,
    );

    // Empty label
    let label = "empty";
    let x = area.x + (area.width.saturating_sub(label.len() as u16)) / 2;
    buf.set_string(x, area.y + 2, label, style);
}
