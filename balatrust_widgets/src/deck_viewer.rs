use std::collections::HashMap;

use balatrust_core::card::{Edition, Enhancement, PlayingCard, Rank, Seal, Suit};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Clear, Widget};

use crate::theme::Theme;

// ─── Rank ordering for display (A high to 2 low, left to right) ──────

const RANKS_HIGH_TO_LOW: [Rank; 13] = [
    Rank::Ace,
    Rank::King,
    Rank::Queen,
    Rank::Jack,
    Rank::Ten,
    Rank::Nine,
    Rank::Eight,
    Rank::Seven,
    Rank::Six,
    Rank::Five,
    Rank::Four,
    Rank::Three,
    Rank::Two,
];

const SUITS_ORDER: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];

// ═══════════════════════════════════════════════════════════════════════
// DeckPreviewWidget — small stacked-card icon for the right sidebar
// ═══════════════════════════════════════════════════════════════════════

/// A small clickable deck preview that looks like a stack of cards.
/// Renders in the right sidebar bottom area.
pub struct DeckPreviewWidget {
    pub total_cards: usize,
    pub remaining: usize,
}

impl DeckPreviewWidget {
    pub fn new(total_cards: usize, remaining: usize) -> Self {
        Self {
            total_cards,
            remaining,
        }
    }

    /// Return the rect where the preview is rendered (for hit-testing).
    /// Call with the same area you pass to render.
    pub fn hit_rect(area: Rect) -> Rect {
        if area.width < 6 || area.height < 6 {
            return Rect::default();
        }
        let w = 10u16.min(area.width);
        let h = 6u16.min(area.height);
        let x = area.x + area.width.saturating_sub(w) / 2;
        let y = area.y;
        Rect::new(x, y, w, h)
    }
}

impl Widget for DeckPreviewWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 6 || area.height < 6 {
            return;
        }

        let w = 10u16.min(area.width);
        let h = 5u16;
        let x = area.x + area.width.saturating_sub(w) / 2;
        let y = area.y;

        let border_style = Style::default().fg(Theme::CARD_BORDER);
        let back_style = Style::default().fg(Theme::CARD_BACK);

        // Draw 3 offset card backs to create a "stack" effect
        // Card 3 (bottom of stack, offset +2,+1)
        if w >= 8 && h + 2 <= area.height {
            draw_mini_card_back(x + 2, y + 1, w - 2, h - 1, buf, border_style, back_style);
        }
        // Card 2 (middle, offset +1,+0.5)
        if w >= 9 {
            draw_mini_card_back(x + 1, y, w - 1, h, buf, border_style, back_style);
        }
        // Card 1 (top, front)
        draw_mini_card_front(x, y, w, h, buf);

        // Count below the stack
        let count_str = format!("{}/{}", self.remaining, self.total_cards);
        let cx = area.x + area.width.saturating_sub(count_str.len() as u16) / 2;
        let cy = y + h;
        if cy < area.bottom() {
            buf.set_string(
                cx,
                cy,
                &count_str,
                Style::default()
                    .fg(Theme::MUTED_TEXT)
                    .add_modifier(Modifier::BOLD),
            );
        }
    }
}

fn draw_mini_card_back(
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    buf: &mut Buffer,
    border: Style,
    fill: Style,
) {
    // Top border
    buf.set_string(x, y, "\u{256d}", border);
    for dx in 1..w.saturating_sub(1) {
        buf.set_string(x + dx, y, "\u{2500}", border);
    }
    buf.set_string(x + w - 1, y, "\u{256e}", border);

    // Middle rows
    for dy in 1..h.saturating_sub(1) {
        buf.set_string(x, y + dy, "\u{2502}", border);
        for dx in 1..w.saturating_sub(1) {
            let ch = if (dx + dy) % 2 == 0 {
                "\u{2593}"
            } else {
                "\u{2591}"
            };
            buf.set_string(x + dx, y + dy, ch, fill);
        }
        buf.set_string(x + w - 1, y + dy, "\u{2502}", border);
    }

    // Bottom border
    let by = y + h - 1;
    buf.set_string(x, by, "\u{2570}", border);
    for dx in 1..w.saturating_sub(1) {
        buf.set_string(x + dx, by, "\u{2500}", border);
    }
    buf.set_string(x + w - 1, by, "\u{256f}", border);
}

fn draw_mini_card_front(x: u16, y: u16, w: u16, h: u16, buf: &mut Buffer) {
    let border = Style::default().fg(Theme::GOLD);

    // Top border
    buf.set_string(x, y, "\u{256d}", border);
    for dx in 1..w.saturating_sub(1) {
        buf.set_string(x + dx, y, "\u{2500}", border);
    }
    buf.set_string(x + w - 1, y, "\u{256e}", border);

    // Middle rows (blank interior)
    for dy in 1..h.saturating_sub(1) {
        buf.set_string(x, y + dy, "\u{2502}", border);
        for dx in 1..w.saturating_sub(1) {
            buf.set_string(x + dx, y + dy, " ", Style::default());
        }
        buf.set_string(x + w - 1, y + dy, "\u{2502}", border);
    }

    // Bottom border
    let by = y + h - 1;
    buf.set_string(x, by, "\u{2570}", border);
    for dx in 1..w.saturating_sub(1) {
        buf.set_string(x + dx, by, "\u{2500}", border);
    }
    buf.set_string(x + w - 1, by, "\u{256f}", border);

    // "DECK" label centered
    let label = "DECK";
    let lx = x + w.saturating_sub(label.len() as u16) / 2;
    let ly = y + h / 2;
    buf.set_string(
        lx,
        ly,
        label,
        Style::default()
            .fg(Theme::GOLD)
            .add_modifier(Modifier::BOLD),
    );
}

// ═══════════════════════════════════════════════════════════════════════
// DeckOverlayWidget — full-screen deck viewer overlay
// ═══════════════════════════════════════════════════════════════════════

/// Full-screen overlay showing the entire deck contents.
pub struct DeckOverlayWidget<'a> {
    pub cards: &'a [PlayingCard],
    pub selected_card: Option<usize>,
}

/// Precomputed stats about the deck for the info panel
struct DeckStats {
    rank_counts: [(Rank, usize); 13],
    suit_counts: [(Suit, usize); 4],
    enhanced_count: usize,
    sealed_count: usize,
    edition_count: usize,
    total: usize,
}

impl DeckStats {
    fn from_cards(cards: &[PlayingCard]) -> Self {
        let mut rank_map: HashMap<Rank, usize> = HashMap::new();
        let mut suit_map: HashMap<Suit, usize> = HashMap::new();
        let mut enhanced = 0usize;
        let mut sealed = 0usize;
        let mut edition = 0usize;

        for card in cards {
            *rank_map.entry(card.rank).or_default() += 1;
            *suit_map.entry(card.suit).or_default() += 1;
            if card.enhancement.is_some() {
                enhanced += 1;
            }
            if card.seal.is_some() {
                sealed += 1;
            }
            if card.edition != Edition::Base {
                edition += 1;
            }
        }

        let rank_counts: [(Rank, usize); 13] =
            RANKS_HIGH_TO_LOW.map(|r| (r, *rank_map.get(&r).unwrap_or(&0)));

        let suit_counts: [(Suit, usize); 4] =
            SUITS_ORDER.map(|s| (s, *suit_map.get(&s).unwrap_or(&0)));

        Self {
            rank_counts,
            suit_counts,
            enhanced_count: enhanced,
            sealed_count: sealed,
            edition_count: edition,
            total: cards.len(),
        }
    }
}

impl<'a> DeckOverlayWidget<'a> {
    pub fn new(cards: &'a [PlayingCard], selected_card: Option<usize>) -> Self {
        Self {
            cards,
            selected_card,
        }
    }

    /// Compute the card grid rects for hit-testing.
    /// Returns a vec of (global_card_index, Rect) for each card cell rendered.
    pub fn card_cell_rects(area: Rect, cards: &[PlayingCard]) -> Vec<(usize, Rect)> {
        let overlay = Self::overlay_rect(area);
        if overlay.width < 20 || overlay.height < 10 {
            return Vec::new();
        }

        let inner = Self::inner_rect(overlay);
        let grid_area = Self::grid_area(inner);

        let mut result = Vec::new();
        let cards_by_suit = Self::group_by_suit(cards);
        let cell_w = Self::cell_width(grid_area);

        let mut y = grid_area.y;
        for &suit in &SUITS_ORDER {
            if y + 1 > grid_area.bottom() {
                break;
            }
            let suit_cards = cards_by_suit.get(&suit).cloned().unwrap_or_default();
            let mut x = grid_area.x + 2; // offset for suit symbol
            for &(global_idx, _card) in &suit_cards {
                if x + cell_w > grid_area.right() {
                    break;
                }
                result.push((global_idx, Rect::new(x, y, cell_w, 1)));
                x += cell_w;
            }
            y += 2; // row + gap
        }
        result
    }

    /// Get the back button rect for hit-testing
    pub fn back_button_rect(area: Rect) -> Rect {
        let overlay = Self::overlay_rect(area);
        if overlay.width < 20 || overlay.height < 10 {
            return Rect::default();
        }
        let btn_h = 3u16;
        Rect::new(
            overlay.x + 1,
            overlay.y + overlay.height - btn_h - 1,
            overlay.width - 2,
            btn_h,
        )
    }

    fn overlay_rect(area: Rect) -> Rect {
        // 90% width, 85% height, centered
        let w = (area.width * 90 / 100).max(40).min(area.width);
        let h = (area.height * 85 / 100).max(20).min(area.height);
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        Rect::new(x, y, w, h)
    }

    fn inner_rect(overlay: Rect) -> Rect {
        Rect::new(
            overlay.x + 2,
            overlay.y + 3, // after top border + title
            overlay.width.saturating_sub(4),
            overlay.height.saturating_sub(7), // border + title + back button
        )
    }

    fn grid_area(inner: Rect) -> Rect {
        // Grid takes the right portion after the info panel
        let info_w = 22u16.min(inner.width / 3);
        Rect::new(
            inner.x + info_w + 1,
            inner.y,
            inner.width.saturating_sub(info_w + 1),
            inner.height,
        )
    }

    fn cell_width(grid_area: Rect) -> u16 {
        // Each card cell is ~4 chars (e.g. "A\u{2660} "), at least 3
        let available = grid_area.width.saturating_sub(2); // suit label
        let per_card = available / 13;
        per_card.max(3).min(5)
    }

    fn group_by_suit(cards: &[PlayingCard]) -> HashMap<Suit, Vec<(usize, PlayingCard)>> {
        let mut map: HashMap<Suit, Vec<(usize, PlayingCard)>> = HashMap::new();
        for (i, card) in cards.iter().enumerate() {
            map.entry(card.suit).or_default().push((i, *card));
        }
        // Sort each suit's cards by rank high to low
        for cards_vec in map.values_mut() {
            cards_vec.sort_by(|a, b| rank_sort_key(b.1.rank).cmp(&rank_sort_key(a.1.rank)));
        }
        map
    }
}

fn rank_sort_key(rank: Rank) -> u8 {
    rank as u8
}

fn suit_color(suit: Suit) -> ratatui::style::Color {
    match suit {
        Suit::Spades => Theme::BLACK_SUIT,
        Suit::Hearts => Theme::RED_SUIT,
        Suit::Clubs => Theme::BLACK_SUIT,
        Suit::Diamonds => Theme::RED_SUIT,
    }
}

fn suit_symbol(suit: Suit) -> &'static str {
    match suit {
        Suit::Spades => "\u{2660}",
        Suit::Hearts => "\u{2665}",
        Suit::Clubs => "\u{2663}",
        Suit::Diamonds => "\u{2666}",
    }
}

impl<'a> Widget for DeckOverlayWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let overlay = Self::overlay_rect(area);
        if overlay.width < 20 || overlay.height < 10 {
            return;
        }

        // Clear the overlay area
        Clear.render(overlay, buf);

        // Draw outer border
        let border_style = Style::default().fg(Theme::CARD_SELECTED);
        draw_overlay_border(overlay, buf, border_style);

        // Title badge: "Full Deck" centered at top
        let title = " Full Deck ";
        let title_x = overlay.x + overlay.width.saturating_sub(title.len() as u16) / 2;
        buf.set_string(
            title_x,
            overlay.y,
            title,
            Style::default()
                .fg(Theme::MULT_COLOR)
                .add_modifier(Modifier::BOLD),
        );

        // Downward arrow above title
        let arrow_x = overlay.x + overlay.width / 2;
        if overlay.y > 0 {
            buf.set_string(
                arrow_x,
                overlay.y + 1,
                "\u{25bc}",
                Style::default().fg(Theme::MULT_COLOR),
            );
        }

        let inner = Self::inner_rect(overlay);
        if inner.width < 10 || inner.height < 4 {
            return;
        }

        let stats = DeckStats::from_cards(self.cards);

        // Split inner into left info panel and right card grid
        let info_w = 22u16.min(inner.width / 3);
        let info_area = Rect::new(inner.x, inner.y, info_w, inner.height);
        let grid_area = Self::grid_area(inner);

        // ─── Left Info Panel ─────────────────────────────────────────
        self.render_info_panel(info_area, buf, &stats);

        // ─── Right Card Grid ─────────────────────────────────────────
        self.render_card_grid(grid_area, buf, &stats);

        // ─── Card Detail Tooltip ─────────────────────────────────────
        if let Some(sel) = self.selected_card {
            if let Some(card) = self.cards.get(sel) {
                self.render_card_tooltip(area, overlay, buf, card);
            }
        }

        // ─── Back Button ─────────────────────────────────────────────
        let btn = Self::back_button_rect(area);
        render_back_button(btn, buf);
    }
}

impl<'a> DeckOverlayWidget<'a> {
    fn render_info_panel(&self, area: Rect, buf: &mut Buffer, stats: &DeckStats) {
        if area.width < 5 || area.height < 4 {
            return;
        }

        let mut y = area.y;

        // Deck total
        let total_str = format!("{} cards", stats.total);
        buf.set_string(
            area.x,
            y,
            &total_str,
            Style::default()
                .fg(Theme::BRIGHT_TEXT)
                .add_modifier(Modifier::BOLD),
        );
        y += 2;

        // Rank counts
        let label_style = Style::default().fg(Theme::MUTED_TEXT);
        let count_style = Style::default()
            .fg(Theme::BRIGHT_TEXT)
            .add_modifier(Modifier::BOLD);

        for &(rank, count) in &stats.rank_counts {
            if y >= area.bottom() {
                break;
            }
            let rank_str = format!("{:>2}", rank.short_name());
            buf.set_string(area.x, y, &rank_str, label_style);
            buf.set_string(area.x + 3, y, &format!("{}", count), count_style);
            y += 1;
        }

        y += 1;
        if y >= area.bottom() {
            return;
        }

        // Section: "Base Cards"
        buf.set_string(
            area.x,
            y,
            "Base Cards",
            Style::default()
                .fg(Theme::GOLD)
                .add_modifier(Modifier::BOLD),
        );
        y += 1;

        // Enhanced / Sealed / Edition counts
        if y < area.bottom() {
            let enh_line = format!(
                "Enh:{} Seal:{} Ed:{}",
                stats.enhanced_count, stats.sealed_count, stats.edition_count
            );
            buf.set_string(area.x, y, &enh_line, Style::default().fg(Theme::MUTED_TEXT));
            y += 1;
        }

        y += 1;
        if y >= area.bottom() {
            return;
        }

        // Suit counts (2 per line)
        for pair in stats.suit_counts.chunks(2) {
            if y >= area.bottom() {
                break;
            }
            let mut x = area.x;
            for &(suit, count) in pair {
                let sym = suit_symbol(suit);
                let color = suit_color(suit);
                buf.set_string(x, y, sym, Style::default().fg(color));
                x += 1;
                let cnt = format!("{:<4}", count);
                buf.set_string(
                    x,
                    y,
                    &cnt,
                    Style::default()
                        .fg(Theme::BRIGHT_TEXT)
                        .add_modifier(Modifier::BOLD),
                );
                x += 5;
            }
            y += 1;
        }
    }

    fn render_card_grid(&self, area: Rect, buf: &mut Buffer, _stats: &DeckStats) {
        if area.width < 10 || area.height < 4 {
            return;
        }

        let cards_by_suit = Self::group_by_suit(self.cards);
        let cell_w = Self::cell_width(area);

        let mut y = area.y;
        for &suit in &SUITS_ORDER {
            if y + 1 > area.bottom() {
                break;
            }

            // Suit label
            let sym = suit_symbol(suit);
            let color = suit_color(suit);
            buf.set_string(y.min(area.y), y, "", Style::default()); // noop positioning
            buf.set_string(area.x, y, sym, Style::default().fg(color));

            let suit_cards = cards_by_suit.get(&suit).cloned().unwrap_or_default();
            let mut x = area.x + 2;
            for &(global_idx, card) in suit_cards.iter() {
                if x + cell_w > area.right() {
                    break;
                }

                let is_selected = self.selected_card == Some(global_idx);
                let rank_str = card.rank.short_name();
                let cell_str = format!("{}{}", rank_str, sym);

                // Determine style based on card properties
                let base_style = if is_selected {
                    Style::default()
                        .fg(Theme::CARD_SELECTED)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else if card.debuffed {
                    Style::default().fg(Theme::DIM_TEXT)
                } else if card.enhancement.is_some() {
                    let enh_color = match card.enhancement {
                        Some(Enhancement::Bonus) => Theme::CHIPS_COLOR,
                        Some(Enhancement::Mult) => Theme::MULT_COLOR,
                        Some(Enhancement::Wild) => Theme::LEGENDARY,
                        Some(Enhancement::Glass) => Theme::BRIGHT_TEXT,
                        Some(Enhancement::Steel) => Theme::DIM_TEXT,
                        Some(Enhancement::Stone) => Theme::MUTED_TEXT,
                        Some(Enhancement::Gold) => Theme::GOLD,
                        Some(Enhancement::Lucky) => Theme::MONEY_COLOR,
                        None => color,
                    };
                    Style::default().fg(enh_color).add_modifier(Modifier::BOLD)
                } else if card.edition != Edition::Base {
                    let ed_style = match card.edition {
                        Edition::Foil => Style::default()
                            .fg(Theme::CHIPS_COLOR)
                            .add_modifier(Modifier::BOLD),
                        Edition::Holographic => Style::default()
                            .fg(Theme::MULT_COLOR)
                            .add_modifier(Modifier::ITALIC),
                        Edition::Polychrome => Style::default()
                            .fg(Theme::LEGENDARY)
                            .add_modifier(Modifier::BOLD),
                        Edition::Base => Style::default().fg(color),
                    };
                    ed_style
                } else {
                    Style::default().fg(color)
                };

                // Seal indicator: small dot after the card
                let seal_char = match card.seal {
                    Some(Seal::Gold) => "\u{2022}", // bullet
                    Some(Seal::Red) => "\u{2022}",
                    Some(Seal::Blue) => "\u{2022}",
                    Some(Seal::Purple) => "\u{2022}",
                    None => "",
                };

                buf.set_string(x, y, &cell_str, base_style);

                if !seal_char.is_empty() {
                    let seal_color = match card.seal {
                        Some(Seal::Gold) => Theme::GOLD,
                        Some(Seal::Red) => Theme::MULT_COLOR,
                        Some(Seal::Blue) => Theme::CHIPS_COLOR,
                        Some(Seal::Purple) => Theme::LEGENDARY,
                        None => Theme::DIM_TEXT,
                    };
                    let seal_x = x + cell_str.len() as u16;
                    if seal_x < area.right() {
                        buf.set_string(seal_x, y, seal_char, Style::default().fg(seal_color));
                    }
                }

                x += cell_w;
            }

            y += 2; // row + gap between suits
        }
    }

    fn render_card_tooltip(
        &self,
        screen: Rect,
        overlay: Rect,
        buf: &mut Buffer,
        card: &PlayingCard,
    ) {
        let name = format!(
            "{} of {}",
            rank_full_name(card.rank),
            suit_full_name(card.suit)
        );
        let chips = format!("+{} Chips", card.chip_value());

        let mut lines: Vec<(String, Style)> = vec![
            (
                name,
                Style::default()
                    .fg(Theme::BRIGHT_TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            (
                chips,
                Style::default()
                    .fg(Theme::CHIPS_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
        ];

        if card.mult_bonus() > 0 {
            lines.push((
                format!("+{} Mult", card.mult_bonus()),
                Style::default()
                    .fg(Theme::MULT_COLOR)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        if card.x_mult() > 1.0 {
            lines.push((
                format!("x{:.1} Mult", card.x_mult()),
                Style::default()
                    .fg(Theme::XMULT_COLOR)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        if let Some(enh) = &card.enhancement {
            let enh_name = match enh {
                Enhancement::Bonus => "Bonus",
                Enhancement::Mult => "Mult",
                Enhancement::Wild => "Wild",
                Enhancement::Glass => "Glass",
                Enhancement::Steel => "Steel",
                Enhancement::Stone => "Stone",
                Enhancement::Gold => "Gold",
                Enhancement::Lucky => "Lucky",
            };
            lines.push((
                format!("Enhancement: {}", enh_name),
                Style::default().fg(Theme::GOLD),
            ));
        }

        if card.edition != Edition::Base {
            let ed_name = match card.edition {
                Edition::Foil => "Foil",
                Edition::Holographic => "Holographic",
                Edition::Polychrome => "Polychrome",
                Edition::Base => "",
            };
            lines.push((
                format!("Edition: {}", ed_name),
                Style::default().fg(Theme::LEGENDARY),
            ));
        }

        if let Some(seal) = &card.seal {
            let seal_name = match seal {
                Seal::Gold => "Gold",
                Seal::Red => "Red",
                Seal::Blue => "Blue",
                Seal::Purple => "Purple",
            };
            lines.push((
                format!("Seal: {}", seal_name),
                Style::default().fg(Theme::GOLD),
            ));
        }

        if card.debuffed {
            lines.push((
                "DEBUFFED".to_string(),
                Style::default()
                    .fg(Theme::DIM_TEXT)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        let content_w = lines
            .iter()
            .map(|(s, _)| s.len() as u16)
            .max()
            .unwrap_or(10)
            + 4;
        let popup_w = content_w.clamp(20, 36);
        let popup_h = lines.len() as u16 + 2; // + border

        // Position tooltip near the center-right of the overlay
        let popup_x = overlay.x + overlay.width.saturating_sub(popup_w + 3);
        let popup_y = overlay.y + 3;

        let popup_x = popup_x
            .max(screen.x)
            .min(screen.right().saturating_sub(popup_w));
        let popup_y = popup_y
            .max(screen.y)
            .min(screen.bottom().saturating_sub(popup_h));

        let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);
        Clear.render(popup_area, buf);

        // Border
        let border_style = Style::default().fg(suit_color(card.suit));
        buf.set_string(popup_area.x, popup_area.y, "\u{256d}", border_style);
        for dx in 1..popup_w.saturating_sub(1) {
            buf.set_string(popup_area.x + dx, popup_area.y, "\u{2500}", border_style);
        }
        buf.set_string(
            popup_area.x + popup_w - 1,
            popup_area.y,
            "\u{256e}",
            border_style,
        );

        for dy in 1..popup_h.saturating_sub(1) {
            buf.set_string(popup_area.x, popup_area.y + dy, "\u{2502}", border_style);
            for dx in 1..popup_w.saturating_sub(1) {
                buf.set_string(popup_area.x + dx, popup_area.y + dy, " ", Style::default());
            }
            buf.set_string(
                popup_area.x + popup_w - 1,
                popup_area.y + dy,
                "\u{2502}",
                border_style,
            );
        }

        let by = popup_area.y + popup_h - 1;
        buf.set_string(popup_area.x, by, "\u{2570}", border_style);
        for dx in 1..popup_w.saturating_sub(1) {
            buf.set_string(popup_area.x + dx, by, "\u{2500}", border_style);
        }
        buf.set_string(popup_area.x + popup_w - 1, by, "\u{256f}", border_style);

        // Content
        for (i, (text, style)) in lines.iter().enumerate() {
            let ly = popup_area.y + 1 + i as u16;
            if ly >= popup_area.bottom().saturating_sub(1) {
                break;
            }
            let lx = popup_area.x + 2;
            let max_len = (popup_w.saturating_sub(4)) as usize;
            let truncated: String = text.chars().take(max_len).collect();
            buf.set_string(lx, ly, &truncated, *style);
        }
    }
}

fn rank_full_name(rank: Rank) -> &'static str {
    match rank {
        Rank::Ace => "Ace",
        Rank::King => "King",
        Rank::Queen => "Queen",
        Rank::Jack => "Jack",
        Rank::Ten => "10",
        Rank::Nine => "9",
        Rank::Eight => "8",
        Rank::Seven => "7",
        Rank::Six => "6",
        Rank::Five => "5",
        Rank::Four => "4",
        Rank::Three => "3",
        Rank::Two => "2",
    }
}

fn suit_full_name(suit: Suit) -> &'static str {
    match suit {
        Suit::Spades => "Spades",
        Suit::Hearts => "Hearts",
        Suit::Clubs => "Clubs",
        Suit::Diamonds => "Diamonds",
    }
}

fn draw_overlay_border(area: Rect, buf: &mut Buffer, style: Style) {
    // Top
    buf.set_string(area.x, area.y, "\u{2554}", style);
    for dx in 1..area.width.saturating_sub(1) {
        buf.set_string(area.x + dx, area.y, "\u{2550}", style);
    }
    buf.set_string(area.x + area.width - 1, area.y, "\u{2557}", style);

    // Sides
    for dy in 1..area.height.saturating_sub(1) {
        buf.set_string(area.x, area.y + dy, "\u{2551}", style);
        // Fill interior with spaces (dark bg)
        for dx in 1..area.width.saturating_sub(1) {
            buf.set_string(area.x + dx, area.y + dy, " ", Style::default());
        }
        buf.set_string(area.x + area.width - 1, area.y + dy, "\u{2551}", style);
    }

    // Bottom
    buf.set_string(area.x, area.y + area.height - 1, "\u{255a}", style);
    for dx in 1..area.width.saturating_sub(1) {
        buf.set_string(area.x + dx, area.y + area.height - 1, "\u{2550}", style);
    }
    buf.set_string(
        area.x + area.width - 1,
        area.y + area.height - 1,
        "\u{255d}",
        style,
    );
}

fn render_back_button(area: Rect, buf: &mut Buffer) {
    if area.width < 6 || area.height < 1 {
        return;
    }

    let btn_style = Style::default()
        .fg(Theme::BRIGHT_TEXT)
        .add_modifier(Modifier::BOLD);
    let bg_style = Style::default().fg(Theme::GOLD);

    // Top border of button
    let top_y = area.y;
    buf.set_string(area.x, top_y, "\u{256d}", bg_style);
    for dx in 1..area.width.saturating_sub(1) {
        buf.set_string(area.x + dx, top_y, "\u{2500}", bg_style);
    }
    buf.set_string(area.x + area.width - 1, top_y, "\u{256e}", bg_style);

    // Middle row with label
    if area.height >= 2 {
        let mid_y = area.y + 1;
        buf.set_string(area.x, mid_y, "\u{2502}", bg_style);
        for dx in 1..area.width.saturating_sub(1) {
            buf.set_string(area.x + dx, mid_y, " ", Style::default());
        }
        buf.set_string(area.x + area.width - 1, mid_y, "\u{2502}", bg_style);

        let label = "Back";
        let lx = area.x + area.width.saturating_sub(label.len() as u16) / 2;
        buf.set_string(lx, mid_y, label, btn_style);
    }

    // Bottom border
    if area.height >= 3 {
        let bot_y = area.y + 2;
        buf.set_string(area.x, bot_y, "\u{2570}", bg_style);
        for dx in 1..area.width.saturating_sub(1) {
            buf.set_string(area.x + dx, bot_y, "\u{2500}", bg_style);
        }
        buf.set_string(area.x + area.width - 1, bot_y, "\u{256f}", bg_style);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// DeckViewerState — shared state for screens to use
// ═══════════════════════════════════════════════════════════════════════

/// Shared state struct that any screen can embed to manage the deck viewer.
pub struct DeckViewerState {
    /// Whether the overlay is currently open
    pub open: bool,
    /// Index of the currently selected card in the full_deck vec (for tooltip)
    pub selected_card: Option<usize>,
    /// Cached full deck snapshot (refreshed when overlay opens)
    pub cached_deck: Vec<PlayingCard>,
    /// Cached card cell rects for hit-testing
    pub card_rects: Vec<(usize, Rect)>,
    /// Cached back button rect
    pub back_rect: Rect,
    /// Cached deck preview rect (right sidebar)
    pub preview_rect: Rect,
}

impl DeckViewerState {
    pub fn new() -> Self {
        Self {
            open: false,
            selected_card: None,
            cached_deck: Vec::new(),
            card_rects: Vec::new(),
            back_rect: Rect::default(),
            preview_rect: Rect::default(),
        }
    }

    /// Open the deck viewer with fresh deck data
    pub fn open(&mut self, full_deck: Vec<PlayingCard>) {
        self.cached_deck = full_deck;
        self.selected_card = None;
        self.open = true;
    }

    /// Close the deck viewer
    pub fn close(&mut self) {
        self.open = false;
        self.selected_card = None;
    }

    /// Toggle open/close
    pub fn toggle(&mut self, full_deck: Vec<PlayingCard>) {
        if self.open {
            self.close();
        } else {
            self.open(full_deck);
        }
    }

    /// Render the preview in the right sidebar bottom area.
    /// Returns the preview rect for hit-testing.
    pub fn render_preview(
        &mut self,
        frame: &mut ratatui::Frame,
        area: Rect,
        total_cards: usize,
        remaining: usize,
    ) {
        let widget = DeckPreviewWidget::new(total_cards, remaining);
        self.preview_rect = DeckPreviewWidget::hit_rect(area);
        frame.render_widget(widget, area);
    }

    /// Render the full-screen overlay if open.
    pub fn render_overlay(&mut self, frame: &mut ratatui::Frame, screen_area: Rect) {
        if !self.open {
            return;
        }

        let widget = DeckOverlayWidget::new(&self.cached_deck, self.selected_card);
        self.card_rects = DeckOverlayWidget::card_cell_rects(screen_area, &self.cached_deck);
        self.back_rect = DeckOverlayWidget::back_button_rect(screen_area);
        frame.render_widget(widget, screen_area);
    }

    /// Handle a key event while the overlay is open.
    /// Returns true if the event was consumed.
    pub fn handle_key(&mut self, code: crossterm::event::KeyCode) -> bool {
        if !self.open {
            return false;
        }

        match code {
            crossterm::event::KeyCode::Esc
            | crossterm::event::KeyCode::Char('v')
            | crossterm::event::KeyCode::Char('V')
            | crossterm::event::KeyCode::Enter => {
                self.close();
                true
            }
            _ => true, // Consume all keys when overlay is open
        }
    }

    /// Handle a mouse click while overlay or preview is active.
    /// Returns Some(true) if the overlay was interacted with (consumed),
    /// Some(false) if the preview was clicked (caller should open),
    /// None if not relevant.
    pub fn handle_mouse_click(&mut self, col: u16, row: u16) -> Option<bool> {
        if self.open {
            // Check back button
            if self.back_rect.width > 0
                && col >= self.back_rect.x
                && col < self.back_rect.x + self.back_rect.width
                && row >= self.back_rect.y
                && row < self.back_rect.y + self.back_rect.height
            {
                self.close();
                return Some(true);
            }

            // Check card cells
            for &(global_idx, rect) in &self.card_rects {
                if rect.width > 0
                    && col >= rect.x
                    && col < rect.x + rect.width
                    && row >= rect.y
                    && row < rect.y + rect.height
                {
                    if self.selected_card == Some(global_idx) {
                        self.selected_card = None;
                    } else {
                        self.selected_card = Some(global_idx);
                    }
                    return Some(true);
                }
            }

            // Click elsewhere in overlay area just deselects
            self.selected_card = None;
            return Some(true);
        }

        // Check preview click
        if self.preview_rect.width > 0
            && col >= self.preview_rect.x
            && col < self.preview_rect.x + self.preview_rect.width
            && row >= self.preview_rect.y
            && row < self.preview_rect.y + self.preview_rect.height
        {
            return Some(false); // Signal: preview was clicked, caller should open
        }

        None
    }
}
