use balatrust_core::card::{Edition, PlayingCard, Rank};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;

use crate::theme::Theme;

/// Width and height of a card in terminal cells
pub const CARD_WIDTH: u16 = 13;
pub const CARD_HEIGHT: u16 = 11;

/// A visual playing card widget
pub struct CardWidget {
    pub card: PlayingCard,
    pub selected: bool,
    pub highlighted: bool,
    pub face_down: bool,
    pub dimmed: bool,
    /// True when this card is actively being scored (bright glow border)
    pub scoring: bool,
}

impl CardWidget {
    pub fn new(card: PlayingCard) -> Self {
        Self {
            card,
            selected: false,
            highlighted: false,
            face_down: false,
            dimmed: false,
            scoring: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted;
        self
    }

    pub fn face_down(mut self, face_down: bool) -> Self {
        self.face_down = face_down;
        self
    }

    pub fn dimmed(mut self, dimmed: bool) -> Self {
        self.dimmed = dimmed;
        self
    }

    pub fn scoring(mut self, scoring: bool) -> Self {
        self.scoring = scoring;
        self
    }

    fn suit_color(&self) -> ratatui::style::Color {
        if self.dimmed || self.card.debuffed {
            Theme::DIM_TEXT
        } else if self.card.suit.is_red() {
            Theme::RED_SUIT
        } else {
            Theme::BLACK_SUIT
        }
    }

    fn border_color(&self) -> ratatui::style::Color {
        if self.scoring {
            Theme::BRIGHT_TEXT
        } else if self.selected {
            Theme::CARD_SELECTED
        } else if self.highlighted {
            Theme::GOLD
        } else {
            Theme::CARD_BORDER
        }
    }

    fn edition_indicator(&self) -> Option<(&str, ratatui::style::Color)> {
        match self.card.edition {
            Edition::Foil => Some(("F", Theme::CHIPS_COLOR)),
            Edition::Holographic => Some(("H", Theme::MULT_COLOR)),
            Edition::Polychrome => Some(("P", Theme::LEGENDARY)),
            Edition::Base => None,
        }
    }
}

impl Widget for CardWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < CARD_WIDTH || area.height < CARD_HEIGHT {
            return;
        }

        let border_color = self.border_color();
        let border_style = Style::default().fg(border_color);

        if self.face_down {
            render_face_down(area, buf, border_style);
            return;
        }

        let suit_color = self.suit_color();
        let rank_style = Style::default().fg(suit_color).add_modifier(Modifier::BOLD);
        let suit_style = Style::default().fg(suit_color);
        let separator_style = Style::default().fg(Theme::CARD_FACE_DIM);

        let rank = self.card.rank.short_name();
        let suit_ch = self.card.suit.symbol();
        let suit_str = suit_ch.to_string();

        // ─── Draw border + fill interior ────────────────────────────

        draw_border(area, buf, border_style, self.selected);

        // ─── Separator lines (rows 2 and 8) ────────────────────────

        for x in 2..(CARD_WIDTH - 2) {
            buf.set_string(area.x + x, area.y + 2, "\u{2508}", separator_style); // ┈
            buf.set_string(
                area.x + x,
                area.y + CARD_HEIGHT - 3,
                "\u{2508}",
                separator_style,
            );
        }

        // ─── Top-left: rank + suit ─────────────────────────────────

        buf.set_string(area.x + 1, area.y + 1, rank, rank_style);
        let suit_offset: u16 = if rank.len() > 1 { 2 } else { 1 };
        buf.set_string(area.x + 1 + suit_offset, area.y + 1, &suit_str, suit_style);

        // ─── Bottom-right: suit + rank (mirrored) ──────────────────

        let br_y = area.y + CARD_HEIGHT - 2;
        let br_x = area.x + CARD_WIDTH - 2;
        if rank.len() > 1 {
            buf.set_string(br_x - 1, br_y, rank, rank_style);
        } else {
            buf.set_string(br_x, br_y, rank, rank_style);
        }
        let br_suit_x = area.x + CARD_WIDTH - 2 - rank.len() as u16;
        buf.set_string(br_suit_x, br_y, &suit_str, suit_style);

        // ─── Pip area ──────────────────────────────────────────────
        //
        // Interior pip grid (inside the separator lines):
        //   Columns: L=area.x+3, C=area.x+6, R=area.x+9
        //   Rows:    r0=area.y+3, r1=area.y+4, r2=area.y+5, r3=area.y+6, r4=area.y+7
        //
        // This maps to real playing card pip positions:
        //   r0 = top row
        //   r1 = upper-mid row
        //   r2 = center row
        //   r3 = lower-mid row
        //   r4 = bottom row

        let col_l = area.x + 3;
        let col_c = area.x + 6;
        let col_r = area.x + 9;

        let row_0 = area.y + 3;
        let row_1 = area.y + 4;
        let row_2 = area.y + 5; // center
        let row_3 = area.y + 6;
        let row_4 = area.y + 7;

        let pip = |x: u16, y: u16, buf: &mut Buffer| {
            buf.set_string(x, y, &suit_str, suit_style);
        };

        let face_letter_style = Style::default().fg(suit_color).add_modifier(Modifier::BOLD);
        let face_dim_style = Style::default().fg(Theme::CARD_FACE_DIM);

        match self.card.rank {
            // ── Ace: single centered pip ─────────────────────────
            Rank::Ace => {
                pip(col_c, row_2, buf);
            }

            // ── 2: top + bottom center ──────────────────────────
            Rank::Two => {
                pip(col_c, row_0, buf);
                pip(col_c, row_4, buf);
            }

            // ── 3: column of 3 ──────────────────────────────────
            Rank::Three => {
                pip(col_c, row_0, buf);
                pip(col_c, row_2, buf);
                pip(col_c, row_4, buf);
            }

            // ── 4: 2x2 corners ──────────────────────────────────
            Rank::Four => {
                pip(col_l, row_0, buf);
                pip(col_r, row_0, buf);
                pip(col_l, row_4, buf);
                pip(col_r, row_4, buf);
            }

            // ── 5: 2x2 + center ────────────────────────────────
            Rank::Five => {
                pip(col_l, row_0, buf);
                pip(col_r, row_0, buf);
                pip(col_c, row_2, buf);
                pip(col_l, row_4, buf);
                pip(col_r, row_4, buf);
            }

            // ── 6: 2 columns of 3 ──────────────────────────────
            Rank::Six => {
                pip(col_l, row_0, buf);
                pip(col_r, row_0, buf);
                pip(col_l, row_2, buf);
                pip(col_r, row_2, buf);
                pip(col_l, row_4, buf);
                pip(col_r, row_4, buf);
            }

            // ── 7: 6-layout + center upper-mid ─────────────────
            Rank::Seven => {
                pip(col_l, row_0, buf);
                pip(col_r, row_0, buf);
                pip(col_c, row_1, buf);
                pip(col_l, row_2, buf);
                pip(col_r, row_2, buf);
                pip(col_l, row_4, buf);
                pip(col_r, row_4, buf);
            }

            // ── 8: 6-layout + center upper + center lower ──────
            Rank::Eight => {
                pip(col_l, row_0, buf);
                pip(col_r, row_0, buf);
                pip(col_c, row_1, buf);
                pip(col_l, row_2, buf);
                pip(col_r, row_2, buf);
                pip(col_c, row_3, buf);
                pip(col_l, row_4, buf);
                pip(col_r, row_4, buf);
            }

            // ── 9: 2 columns of 4 + center ─────────────────────
            Rank::Nine => {
                pip(col_l, row_0, buf);
                pip(col_r, row_0, buf);
                pip(col_l, row_1, buf);
                pip(col_r, row_1, buf);
                pip(col_c, row_2, buf);
                pip(col_l, row_3, buf);
                pip(col_r, row_3, buf);
                pip(col_l, row_4, buf);
                pip(col_r, row_4, buf);
            }

            // ── 10: 2 columns of 4 + center upper + center lower
            Rank::Ten => {
                pip(col_l, row_0, buf);
                pip(col_r, row_0, buf);
                pip(col_c, row_1, buf);
                pip(col_l, row_1, buf);
                pip(col_r, row_1, buf);
                pip(col_l, row_3, buf);
                pip(col_r, row_3, buf);
                pip(col_c, row_3, buf);
                pip(col_l, row_4, buf);
                pip(col_r, row_4, buf);
            }

            // ── Face cards: decorative centered layout ──────────
            Rank::Jack => {
                buf.set_string(col_c, row_0, "\u{2654}", face_dim_style); // ♔
                pip(col_l, row_1, buf);
                buf.set_string(col_c, row_2, "J", face_letter_style);
                pip(col_r, row_3, buf);
                buf.set_string(col_c, row_4, "\u{2654}", face_dim_style);
            }
            Rank::Queen => {
                buf.set_string(col_c, row_0, "\u{2655}", face_dim_style); // ♕
                pip(col_r, row_1, buf);
                buf.set_string(col_c, row_2, "Q", face_letter_style);
                pip(col_l, row_3, buf);
                buf.set_string(col_c, row_4, "\u{2655}", face_dim_style);
            }
            Rank::King => {
                buf.set_string(col_c, row_0, "\u{2654}", face_dim_style); // ♔
                pip(col_l, row_1, buf);
                pip(col_r, row_1, buf);
                buf.set_string(col_c, row_2, "K", face_letter_style);
                pip(col_l, row_3, buf);
                pip(col_r, row_3, buf);
                buf.set_string(col_c, row_4, "\u{2654}", face_dim_style);
            }
        }

        // ─── Edition indicator (top-right corner) ───────────────────

        if let Some((indicator, color)) = self.edition_indicator() {
            let ind_style = Style::default().fg(color).add_modifier(Modifier::BOLD);
            buf.set_string(area.x + CARD_WIDTH - 2, area.y + 1, indicator, ind_style);
        }

        // ─── Enhancement indicator (bottom-left) ────────────────────

        if let Some(enh) = &self.card.enhancement {
            let (symbol, color) = match enh {
                balatrust_core::card::Enhancement::Bonus => ("+", Theme::CHIPS_COLOR),
                balatrust_core::card::Enhancement::Mult => ("x", Theme::MULT_COLOR),
                balatrust_core::card::Enhancement::Wild => ("W", Theme::LEGENDARY),
                balatrust_core::card::Enhancement::Glass => ("G", Theme::BRIGHT_TEXT),
                balatrust_core::card::Enhancement::Steel => ("S", Theme::DIM_TEXT),
                balatrust_core::card::Enhancement::Stone => ("O", Theme::MUTED_TEXT),
                balatrust_core::card::Enhancement::Gold => ("$", Theme::GOLD),
                balatrust_core::card::Enhancement::Lucky => ("L", Theme::MONEY_COLOR),
            };
            let enh_style = Style::default().fg(color);
            buf.set_string(area.x + 1, area.y + CARD_HEIGHT - 2, symbol, enh_style);
        }
    }
}

/// Draw the card border and fill interior with spaces
fn draw_border(area: Rect, buf: &mut Buffer, border_style: Style, selected: bool) {
    if selected {
        // Double-line border for selected cards
        buf.set_string(area.x, area.y, "\u{2554}", border_style);
        for x in 1..CARD_WIDTH - 1 {
            buf.set_string(area.x + x, area.y, "\u{2550}", border_style);
        }
        buf.set_string(area.x + CARD_WIDTH - 1, area.y, "\u{2557}", border_style);

        for y in 1..CARD_HEIGHT - 1 {
            buf.set_string(area.x, area.y + y, "\u{2551}", border_style);
            buf.set_string(
                area.x + CARD_WIDTH - 1,
                area.y + y,
                "\u{2551}",
                border_style,
            );
            for x in 1..CARD_WIDTH - 1 {
                buf.set_string(area.x + x, area.y + y, " ", Style::default());
            }
        }

        buf.set_string(area.x, area.y + CARD_HEIGHT - 1, "\u{255a}", border_style);
        for x in 1..CARD_WIDTH - 1 {
            buf.set_string(
                area.x + x,
                area.y + CARD_HEIGHT - 1,
                "\u{2550}",
                border_style,
            );
        }
        buf.set_string(
            area.x + CARD_WIDTH - 1,
            area.y + CARD_HEIGHT - 1,
            "\u{255d}",
            border_style,
        );
    } else {
        // Rounded corners for normal cards
        buf.set_string(area.x, area.y, "\u{256d}", border_style);
        for x in 1..CARD_WIDTH - 1 {
            buf.set_string(area.x + x, area.y, "\u{2500}", border_style);
        }
        buf.set_string(area.x + CARD_WIDTH - 1, area.y, "\u{256e}", border_style);

        for y in 1..CARD_HEIGHT - 1 {
            buf.set_string(area.x, area.y + y, "\u{2502}", border_style);
            buf.set_string(
                area.x + CARD_WIDTH - 1,
                area.y + y,
                "\u{2502}",
                border_style,
            );
            for x in 1..CARD_WIDTH - 1 {
                buf.set_string(area.x + x, area.y + y, " ", Style::default());
            }
        }

        buf.set_string(area.x, area.y + CARD_HEIGHT - 1, "\u{2570}", border_style);
        for x in 1..CARD_WIDTH - 1 {
            buf.set_string(
                area.x + x,
                area.y + CARD_HEIGHT - 1,
                "\u{2500}",
                border_style,
            );
        }
        buf.set_string(
            area.x + CARD_WIDTH - 1,
            area.y + CARD_HEIGHT - 1,
            "\u{256f}",
            border_style,
        );
    }
}

fn render_face_down(area: Rect, buf: &mut Buffer, border_style: Style) {
    let fill_style = Style::default().fg(Theme::CARD_BACK);

    // Top border
    buf.set_string(area.x, area.y, "\u{256d}", border_style);
    for x in 1..CARD_WIDTH - 1 {
        buf.set_string(area.x + x, area.y, "\u{2500}", border_style);
    }
    buf.set_string(area.x + CARD_WIDTH - 1, area.y, "\u{256e}", border_style);

    // Fill with pattern
    for y in 1..CARD_HEIGHT - 1 {
        buf.set_string(area.x, area.y + y, "\u{2502}", border_style);
        for x in 1..CARD_WIDTH - 1 {
            let pattern = if (x + y) % 2 == 0 {
                "\u{2593}"
            } else {
                "\u{2591}"
            };
            buf.set_string(area.x + x, area.y + y, pattern, fill_style);
        }
        buf.set_string(
            area.x + CARD_WIDTH - 1,
            area.y + y,
            "\u{2502}",
            border_style,
        );
    }

    // Bottom border
    buf.set_string(area.x, area.y + CARD_HEIGHT - 1, "\u{2570}", border_style);
    for x in 1..CARD_WIDTH - 1 {
        buf.set_string(
            area.x + x,
            area.y + CARD_HEIGHT - 1,
            "\u{2500}",
            border_style,
        );
    }
    buf.set_string(
        area.x + CARD_WIDTH - 1,
        area.y + CARD_HEIGHT - 1,
        "\u{256f}",
        border_style,
    );
}
