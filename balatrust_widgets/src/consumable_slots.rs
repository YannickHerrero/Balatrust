use balatrust_core::consumable::{Consumable, ConsumableType};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;

use crate::theme::Theme;

const SLOT_WIDTH: u16 = 10;
const SLOT_HEIGHT: u16 = 5;

/// Right sidebar widget showing consumable card slots.
/// Displays owned consumables and empty slots with a counter.
pub struct ConsumableSlotsWidget<'a> {
    pub consumables: &'a [Consumable],
    pub max_consumables: u8,
}

impl<'a> ConsumableSlotsWidget<'a> {
    pub fn new(consumables: &'a [Consumable], max_consumables: u8) -> Self {
        Self {
            consumables,
            max_consumables,
        }
    }
}

impl<'a> Widget for ConsumableSlotsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 6 || area.height < 3 {
            return;
        }

        let mut y = area.y;

        // Counter header: "0/2"
        let counter = format!("{}/{}", self.consumables.len(), self.max_consumables);
        let x = area.x + area.width.saturating_sub(counter.len() as u16) / 2;
        buf.set_string(
            x,
            y,
            &counter,
            Style::default()
                .fg(Theme::MUTED_TEXT)
                .add_modifier(Modifier::BOLD),
        );
        y += 1;

        // Render each slot
        for i in 0..self.max_consumables as usize {
            if y + SLOT_HEIGHT > area.bottom() {
                break;
            }

            let slot_w = SLOT_WIDTH.min(area.width);
            let slot_x = area.x + area.width.saturating_sub(slot_w) / 2;
            let slot_area = Rect::new(slot_x, y, slot_w, SLOT_HEIGHT);

            if let Some(consumable) = self.consumables.get(i) {
                render_consumable_card(consumable, slot_area, buf);
            } else {
                render_empty_slot(slot_area, buf);
            }

            y += SLOT_HEIGHT + 1; // +1 gap between slots
        }
    }
}

fn render_consumable_card(consumable: &Consumable, area: Rect, buf: &mut Buffer) {
    let color = match consumable.consumable_type {
        ConsumableType::Planet(_) => Theme::CHIPS_COLOR,
        ConsumableType::Tarot(_) => Theme::LEGENDARY,
    };

    let border_style = Style::default().fg(color);

    // Top border
    buf.set_string(area.x, area.y, "\u{256d}", border_style);
    for x in 1..area.width.saturating_sub(1) {
        buf.set_string(area.x + x, area.y, "\u{2500}", border_style);
    }
    buf.set_string(area.x + area.width - 1, area.y, "\u{256e}", border_style);

    // Middle rows
    for row in 1..area.height.saturating_sub(1) {
        buf.set_string(area.x, area.y + row, "\u{2502}", border_style);
        for x in 1..area.width.saturating_sub(1) {
            buf.set_string(area.x + x, area.y + row, " ", Style::default());
        }
        buf.set_string(
            area.x + area.width - 1,
            area.y + row,
            "\u{2502}",
            border_style,
        );
    }

    // Bottom border
    let by = area.y + area.height - 1;
    buf.set_string(area.x, by, "\u{2570}", border_style);
    for x in 1..area.width.saturating_sub(1) {
        buf.set_string(area.x + x, by, "\u{2500}", border_style);
    }
    buf.set_string(area.x + area.width - 1, by, "\u{256f}", border_style);

    // Name (truncated)
    let name = consumable.consumable_type.name();
    let max_len = (area.width - 2) as usize;
    let display_name: String = name.chars().take(max_len).collect();
    let name_x = area.x + 1 + (max_len as u16).saturating_sub(display_name.len() as u16) / 2;
    buf.set_string(
        name_x,
        area.y + 1,
        &display_name,
        Style::default()
            .fg(Theme::BRIGHT_TEXT)
            .add_modifier(Modifier::BOLD),
    );

    // Type label
    let type_label = match consumable.consumable_type {
        ConsumableType::Planet(_) => "Planet",
        ConsumableType::Tarot(_) => "Tarot",
    };
    let type_display: String = type_label.chars().take(max_len).collect();
    let type_x = area.x + 1 + (max_len as u16).saturating_sub(type_display.len() as u16) / 2;
    buf.set_string(
        type_x,
        area.y + 2,
        &type_display,
        Style::default().fg(color),
    );
}

fn render_empty_slot(area: Rect, buf: &mut Buffer) {
    let style = Style::default().fg(Theme::DIM_TEXT);

    // Top border
    buf.set_string(area.x, area.y, "\u{256d}", style);
    for x in 1..area.width.saturating_sub(1) {
        buf.set_string(area.x + x, area.y, "\u{2500}", style);
    }
    buf.set_string(area.x + area.width - 1, area.y, "\u{256e}", style);

    // Middle rows
    for row in 1..area.height.saturating_sub(1) {
        buf.set_string(area.x, area.y + row, "\u{2502}", style);
        for x in 1..area.width.saturating_sub(1) {
            buf.set_string(area.x + x, area.y + row, " ", Style::default());
        }
        buf.set_string(area.x + area.width - 1, area.y + row, "\u{2502}", style);
    }

    // Bottom border
    let by = area.y + area.height - 1;
    buf.set_string(area.x, by, "\u{2570}", style);
    for x in 1..area.width.saturating_sub(1) {
        buf.set_string(area.x + x, by, "\u{2500}", style);
    }
    buf.set_string(area.x + area.width - 1, by, "\u{256f}", style);

    // Empty label
    let label = "empty";
    let max_len = (area.width - 2) as usize;
    let x = area.x + 1 + (max_len as u16).saturating_sub(label.len() as u16) / 2;
    buf.set_string(x, area.y + 2, label, style);
}
