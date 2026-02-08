use balatrust_core::consumable::ConsumableType;
use balatrust_core::joker::JokerRarity;
use balatrust_core::shop::ShopItem;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;

use crate::theme::Theme;

/// Width of a single shop item card
const ITEM_WIDTH: u16 = 22;
/// Height of a single shop item card (including price tag row above)
const ITEM_CARD_HEIGHT: u16 = 6;
/// Height reserved for the price tag above the card
const PRICE_TAG_HEIGHT: u16 = 1;
/// Width of the left button column
const BUTTON_COL_WIDTH: u16 = 16;

/// Shop interior panel widget.
///
/// Renders the shop panel with:
/// - Left column: "Next Round" and "Reroll $X" buttons
/// - Right area: shop item cards with price tags
/// - Bottom row: placeholder empty slots
pub struct ShopPanelWidget<'a> {
    pub items: &'a [ShopItem],
    pub money: u32,
    pub reroll_cost: u32,
    pub selected_item: Option<usize>,
}

impl<'a> ShopPanelWidget<'a> {
    pub fn new(
        items: &'a [ShopItem],
        money: u32,
        reroll_cost: u32,
        selected_item: Option<usize>,
    ) -> Self {
        Self {
            items,
            money,
            reroll_cost,
            selected_item,
        }
    }

    // ─── Hit Testing ──────────────────────────────────────────────────

    /// Get the Rect for the "Next Round" button given the panel area.
    pub fn next_round_rect(area: Rect) -> Rect {
        let inner = Self::inner_rect(area);
        let cols = Layout::horizontal([Constraint::Length(BUTTON_COL_WIDTH), Constraint::Min(0)])
            .split(inner);

        let btn_col = cols[0];
        let buttons = Layout::vertical([
            Constraint::Length(3), // Next Round button
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Reroll button
            Constraint::Min(0),
        ])
        .split(btn_col);

        buttons[0]
    }

    /// Get the Rect for the "Reroll" button given the panel area.
    pub fn reroll_rect(area: Rect) -> Rect {
        let inner = Self::inner_rect(area);
        let cols = Layout::horizontal([Constraint::Length(BUTTON_COL_WIDTH), Constraint::Min(0)])
            .split(inner);

        let btn_col = cols[0];
        let buttons = Layout::vertical([
            Constraint::Length(3), // Next Round button
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Reroll button
            Constraint::Min(0),
        ])
        .split(btn_col);

        buttons[2]
    }

    /// Get the Rects for each shop item card given the panel area.
    pub fn item_rects(area: Rect, item_count: usize) -> Vec<Rect> {
        let inner = Self::inner_rect(area);
        let cols = Layout::horizontal([Constraint::Length(BUTTON_COL_WIDTH), Constraint::Min(0)])
            .split(inner);

        let items_area = cols[1];
        let mut rects = Vec::new();

        if item_count == 0 || items_area.width < ITEM_WIDTH {
            return rects;
        }

        let spacing = 2u16;
        let total_w =
            item_count as u16 * ITEM_WIDTH + (item_count as u16).saturating_sub(1) * spacing;
        let start_x = items_area.x + items_area.width.saturating_sub(total_w) / 2;
        let y = items_area.y + PRICE_TAG_HEIGHT;

        for i in 0..item_count {
            let x = start_x + i as u16 * (ITEM_WIDTH + spacing);
            let card_area = Rect::new(x, y, ITEM_WIDTH, ITEM_CARD_HEIGHT);
            if card_area.right() <= items_area.right() {
                rects.push(card_area);
            } else {
                rects.push(Rect::default());
            }
        }

        rects
    }

    fn inner_rect(area: Rect) -> Rect {
        // Margins inside the outer panel border
        let h_margin = 1u16;
        let v_margin = 1u16;
        Rect::new(
            area.x + h_margin + 1, // +1 for border
            area.y + v_margin + 1, // +1 for border
            area.width.saturating_sub((h_margin + 1) * 2),
            area.height.saturating_sub((v_margin + 1) * 2),
        )
    }
}

impl<'a> Widget for ShopPanelWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 30 || area.height < 10 {
            return;
        }

        // Outer panel border (double, red)
        let border_style = Style::default().fg(Theme::MULT_COLOR);
        self.render_outer_border(area, buf, border_style);

        let inner = Self::inner_rect(area);
        if inner.width < 20 || inner.height < 6 {
            return;
        }

        // Split: button column | items area
        let cols = Layout::horizontal([Constraint::Length(BUTTON_COL_WIDTH), Constraint::Min(0)])
            .split(inner);

        // ═══ Left column: buttons ═══
        self.render_buttons(cols[0], buf);

        // ═══ Right area: items + empty slots ═══
        self.render_items_area(cols[1], buf);
    }
}

impl<'a> ShopPanelWidget<'a> {
    fn render_outer_border(&self, area: Rect, buf: &mut Buffer, style: Style) {
        // Top-left
        buf.set_string(area.x, area.y, "\u{2554}", style); // ╔
                                                           // Top edge
        for x in 1..area.width.saturating_sub(1) {
            buf.set_string(area.x + x, area.y, "\u{2550}", style); // ═
        }
        // Top-right
        buf.set_string(area.x + area.width - 1, area.y, "\u{2557}", style); // ╗

        // Side edges
        for y in 1..area.height.saturating_sub(1) {
            buf.set_string(area.x, area.y + y, "\u{2551}", style); // ║
            buf.set_string(area.x + area.width - 1, area.y + y, "\u{2551}", style);
            // ║
        }

        // Bottom-left
        buf.set_string(area.x, area.y + area.height - 1, "\u{255a}", style); // ╚
                                                                             // Bottom edge
        for x in 1..area.width.saturating_sub(1) {
            buf.set_string(area.x + x, area.y + area.height - 1, "\u{2550}", style);
            // ═
        }
        // Bottom-right
        buf.set_string(
            area.x + area.width - 1,
            area.y + area.height - 1,
            "\u{255d}",
            style,
        ); // ╝
    }

    fn render_buttons(&self, area: Rect, buf: &mut Buffer) {
        let buttons = Layout::vertical([
            Constraint::Length(3), // Next Round button
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Reroll button
            Constraint::Min(0),
        ])
        .split(area);

        // "Next Round" button (red border)
        self.render_button(
            buttons[0],
            buf,
            "Next Round",
            Theme::MULT_COLOR,
            true, // always available
        );

        // "Reroll $X" button (green border)
        let can_reroll = self.money >= self.reroll_cost;
        self.render_button(
            buttons[2],
            buf,
            &format!("Reroll ${}", self.reroll_cost),
            if can_reroll {
                Theme::MONEY_COLOR
            } else {
                Theme::DIM_TEXT
            },
            can_reroll,
        );
    }

    fn render_button(
        &self,
        area: Rect,
        buf: &mut Buffer,
        label: &str,
        color: ratatui::style::Color,
        _enabled: bool,
    ) {
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

    fn render_items_area(&self, area: Rect, buf: &mut Buffer) {
        if area.width < ITEM_WIDTH || area.height < 4 {
            return;
        }

        // Split into top row (items) and bottom row (empty placeholder slots)
        let rows = Layout::vertical([
            Constraint::Length(PRICE_TAG_HEIGHT + ITEM_CARD_HEIGHT), // Items row
            Constraint::Length(1),                                   // Separator
            Constraint::Min(0),                                      // Bottom row (placeholders)
        ])
        .split(area);

        // Render items in top row
        if self.items.is_empty() {
            let text = "No items - Reroll?";
            let x = rows[0].x + rows[0].width.saturating_sub(text.len() as u16) / 2;
            let y = rows[0].y + rows[0].height / 2;
            buf.set_string(x, y, text, Style::default().fg(Theme::DIM_TEXT));
        } else {
            let spacing = 2u16;
            let count = self.items.len() as u16;
            let total_w = count * ITEM_WIDTH + (count.saturating_sub(1)) * spacing;
            let start_x = area.x + area.width.saturating_sub(total_w) / 2;

            for (i, item) in self.items.iter().enumerate() {
                let x = start_x + i as u16 * (ITEM_WIDTH + spacing);
                if x + ITEM_WIDTH > area.right() {
                    break;
                }

                // Price tag above the card
                let price_str = format!("${}", item.price());
                let price_color = if self.money >= item.price() {
                    Theme::MONEY_COLOR
                } else {
                    Theme::MULT_COLOR
                };
                let price_x = x + ITEM_WIDTH.saturating_sub(price_str.len() as u16) / 2;
                buf.set_string(
                    price_x,
                    rows[0].y,
                    &price_str,
                    Style::default()
                        .fg(price_color)
                        .add_modifier(Modifier::BOLD),
                );

                // Card below the price tag
                let card_area = Rect::new(
                    x,
                    rows[0].y + PRICE_TAG_HEIGHT,
                    ITEM_WIDTH,
                    ITEM_CARD_HEIGHT,
                );
                let is_selected = self.selected_item == Some(i);
                self.render_item_card(item, card_area, buf, is_selected);
            }
        }

        // Separator
        if rows[1].height >= 1 {
            let sep: String = "\u{2500}".repeat(area.width.saturating_sub(2) as usize);
            buf.set_string(
                rows[1].x + 1,
                rows[1].y,
                &sep,
                Style::default().fg(Theme::DIM_TEXT),
            );
        }

        // Bottom row: 2 placeholder empty slots
        if rows[2].height >= 5 {
            let slot_w = ITEM_WIDTH;
            let slot_h = 5u16.min(rows[2].height);
            let slot_spacing = 2u16;
            let total_w = 2 * slot_w + slot_spacing;
            let start_x = rows[2].x + rows[2].width.saturating_sub(total_w) / 2;

            for i in 0..2u16 {
                let sx = start_x + i * (slot_w + slot_spacing);
                if sx + slot_w > area.right() {
                    break;
                }
                let slot_area = Rect::new(sx, rows[2].y, slot_w, slot_h);
                self.render_empty_slot(slot_area, buf, if i == 0 { "Voucher" } else { "Pack" });
            }
        }
    }

    fn render_item_card(&self, item: &ShopItem, area: Rect, buf: &mut Buffer, selected: bool) {
        let border_color = if selected {
            Theme::CARD_SELECTED
        } else {
            Theme::CARD_BORDER
        };
        let border_style = Style::default().fg(border_color);

        // Draw rounded border
        if selected {
            // Double border for selected
            buf.set_string(area.x, area.y, "\u{2554}", border_style);
            for x in 1..area.width.saturating_sub(1) {
                buf.set_string(area.x + x, area.y, "\u{2550}", border_style);
            }
            buf.set_string(area.x + area.width - 1, area.y, "\u{2557}", border_style);

            for y in 1..area.height.saturating_sub(1) {
                buf.set_string(area.x, area.y + y, "\u{2551}", border_style);
                buf.set_string(
                    area.x + area.width - 1,
                    area.y + y,
                    "\u{2551}",
                    border_style,
                );
                for x in 1..area.width.saturating_sub(1) {
                    buf.set_string(area.x + x, area.y + y, " ", Style::default());
                }
            }

            let by = area.y + area.height - 1;
            buf.set_string(area.x, by, "\u{255a}", border_style);
            for x in 1..area.width.saturating_sub(1) {
                buf.set_string(area.x + x, by, "\u{2550}", border_style);
            }
            buf.set_string(area.x + area.width - 1, by, "\u{255d}", border_style);
        } else {
            // Rounded border
            buf.set_string(area.x, area.y, "\u{256d}", border_style);
            for x in 1..area.width.saturating_sub(1) {
                buf.set_string(area.x + x, area.y, "\u{2500}", border_style);
            }
            buf.set_string(area.x + area.width - 1, area.y, "\u{256e}", border_style);

            for y in 1..area.height.saturating_sub(1) {
                buf.set_string(area.x, area.y + y, "\u{2502}", border_style);
                buf.set_string(
                    area.x + area.width - 1,
                    area.y + y,
                    "\u{2502}",
                    border_style,
                );
                for x in 1..area.width.saturating_sub(1) {
                    buf.set_string(area.x + x, area.y + y, " ", Style::default());
                }
            }

            let by = area.y + area.height - 1;
            buf.set_string(area.x, by, "\u{2570}", border_style);
            for x in 1..area.width.saturating_sub(1) {
                buf.set_string(area.x + x, by, "\u{2500}", border_style);
            }
            buf.set_string(area.x + area.width - 1, by, "\u{256f}", border_style);
        }

        let max_len = (area.width.saturating_sub(2)) as usize;
        let mut y = area.y + 1;

        // Item name (colored by rarity/type)
        let name = item.name();
        let display_name: String = name.chars().take(max_len).collect();
        let name_color = match item {
            ShopItem::JokerItem(j) => match j.joker_type.rarity() {
                JokerRarity::Common => Theme::COMMON,
                JokerRarity::Uncommon => Theme::UNCOMMON,
                JokerRarity::Rare => Theme::RARE,
                JokerRarity::Legendary => Theme::LEGENDARY,
            },
            ShopItem::ConsumableItem(c) => match c.consumable_type {
                ConsumableType::Planet(_) => Theme::CHIPS_COLOR,
                ConsumableType::Tarot(_) => Theme::LEGENDARY,
            },
        };
        buf.set_string(
            area.x + 1,
            y,
            &display_name,
            Style::default().fg(name_color).add_modifier(Modifier::BOLD),
        );
        y += 1;

        // Type label
        if y < area.y + area.height - 1 {
            let type_label = match item {
                ShopItem::JokerItem(_) => "Joker",
                ShopItem::ConsumableItem(c) => match c.consumable_type {
                    ConsumableType::Planet(_) => "Planet",
                    ConsumableType::Tarot(_) => "Tarot",
                },
            };
            buf.set_string(
                area.x + 1,
                y,
                type_label,
                Style::default().fg(Theme::DIM_TEXT),
            );
            y += 1;
        }

        // Description (truncated, may span 2 lines)
        if y < area.y + area.height - 1 {
            let desc = item.description();
            let display_desc: String = desc.chars().take(max_len).collect();
            buf.set_string(
                area.x + 1,
                y,
                &display_desc,
                Style::default().fg(Theme::MUTED_TEXT),
            );
            y += 1;

            // Second line of description if needed
            if y < area.y + area.height - 1 && desc.len() > max_len {
                let second_line: String = desc.chars().skip(max_len).take(max_len).collect();
                buf.set_string(
                    area.x + 1,
                    y,
                    &second_line,
                    Style::default().fg(Theme::MUTED_TEXT),
                );
            }
        }
    }

    fn render_empty_slot(&self, area: Rect, buf: &mut Buffer, label: &str) {
        let style = Style::default().fg(Theme::DIM_TEXT);

        // Rounded border
        buf.set_string(area.x, area.y, "\u{256d}", style);
        for x in 1..area.width.saturating_sub(1) {
            buf.set_string(area.x + x, area.y, "\u{2500}", style);
        }
        buf.set_string(area.x + area.width - 1, area.y, "\u{256e}", style);

        for y in 1..area.height.saturating_sub(1) {
            buf.set_string(area.x, area.y + y, "\u{2502}", style);
            buf.set_string(area.x + area.width - 1, area.y + y, "\u{2502}", style);
            for x in 1..area.width.saturating_sub(1) {
                buf.set_string(area.x + x, area.y + y, " ", Style::default());
            }
        }

        let by = area.y + area.height - 1;
        buf.set_string(area.x, by, "\u{2570}", style);
        for x in 1..area.width.saturating_sub(1) {
            buf.set_string(area.x + x, by, "\u{2500}", style);
        }
        buf.set_string(area.x + area.width - 1, by, "\u{256f}", style);

        // Label centered
        let max_len = (area.width.saturating_sub(2)) as usize;
        let display: String = label.chars().take(max_len).collect();
        let x = area.x + 1 + (max_len as u16).saturating_sub(display.len() as u16) / 2;
        buf.set_string(x, area.y + area.height / 2, &display, style);
    }
}
