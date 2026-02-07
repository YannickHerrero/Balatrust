use balatrust_core::card::{Edition, PlayingCard};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;

use crate::theme::Theme;

/// Width and height of a card in terminal cells
pub const CARD_WIDTH: u16 = 9;
pub const CARD_HEIGHT: u16 = 7;

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
            Theme::BRIGHT_TEXT // Bright white glow when scoring
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

        let rank = self.card.rank.short_name();
        let suit = self.card.suit.symbol();

        // Selected cards use double borders
        if self.selected {
            // Top border
            buf.set_string(area.x, area.y, "\u{2554}", border_style); // ╔
            for x in 1..CARD_WIDTH - 1 {
                buf.set_string(area.x + x, area.y, "\u{2550}", border_style); // ═
            }
            buf.set_string(area.x + CARD_WIDTH - 1, area.y, "\u{2557}", border_style); // ╗

            // Side borders
            for y in 1..CARD_HEIGHT - 1 {
                buf.set_string(area.x, area.y + y, "\u{2551}", border_style); // ║
                buf.set_string(
                    area.x + CARD_WIDTH - 1,
                    area.y + y,
                    "\u{2551}",
                    border_style,
                );
                // Fill interior
                for x in 1..CARD_WIDTH - 1 {
                    buf.set_string(area.x + x, area.y + y, " ", Style::default());
                }
            }

            // Bottom border
            buf.set_string(area.x, area.y + CARD_HEIGHT - 1, "\u{255a}", border_style); // ╚
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
            ); // ╝
        } else {
            // Rounded corners for normal cards
            // Top border
            buf.set_string(area.x, area.y, "\u{256d}", border_style); // ╭
            for x in 1..CARD_WIDTH - 1 {
                buf.set_string(area.x + x, area.y, "\u{2500}", border_style); // ─
            }
            buf.set_string(area.x + CARD_WIDTH - 1, area.y, "\u{256e}", border_style); // ╮

            // Side borders
            for y in 1..CARD_HEIGHT - 1 {
                buf.set_string(area.x, area.y + y, "\u{2502}", border_style); // │
                buf.set_string(
                    area.x + CARD_WIDTH - 1,
                    area.y + y,
                    "\u{2502}",
                    border_style,
                );
                // Fill interior
                for x in 1..CARD_WIDTH - 1 {
                    buf.set_string(area.x + x, area.y + y, " ", Style::default());
                }
            }

            // Bottom border
            buf.set_string(area.x, area.y + CARD_HEIGHT - 1, "\u{2570}", border_style); // ╰
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
            ); // ╯
        }

        // Top-left: rank and suit
        let rank_x = area.x + 1;
        let rank_y = area.y + 1;
        buf.set_string(rank_x, rank_y, rank, rank_style);
        // Suit next to rank (or below for "10")
        let suit_x = if rank.len() > 1 {
            rank_x + 2
        } else {
            rank_x + 1
        };
        buf.set_string(suit_x, rank_y, &suit.to_string(), suit_style);

        // Center suit (large)
        let center_x = area.x + CARD_WIDTH / 2;
        let center_y = area.y + CARD_HEIGHT / 2;
        buf.set_string(center_x, center_y, &suit.to_string(), suit_style);

        // Bottom-right: suit and rank
        let br_y = area.y + CARD_HEIGHT - 2;
        let br_x = area.x + CARD_WIDTH - 2;
        if rank.len() > 1 {
            buf.set_string(br_x - 1, br_y, rank, rank_style);
        } else {
            buf.set_string(br_x, br_y, rank, rank_style);
        }
        let br_suit_x = area.x + CARD_WIDTH - 2 - rank.len() as u16;
        buf.set_string(br_suit_x, br_y, &suit.to_string(), suit_style);

        // Edition indicator (top-right corner)
        if let Some((indicator, color)) = self.edition_indicator() {
            let ind_style = Style::default().fg(color).add_modifier(Modifier::BOLD);
            buf.set_string(area.x + CARD_WIDTH - 2, area.y + 1, indicator, ind_style);
        }

        // Enhancement indicator (bottom-left)
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
            }; // ▓ ░
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
