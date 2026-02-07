use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

use crate::theme::Theme;

/// Animated background widget that creates a subtle CRT-like felt effect
pub struct BackgroundWidget {
    pub tick: u64,
}

impl BackgroundWidget {
    pub fn new(tick: u64) -> Self {
        Self { tick }
    }
}

impl Widget for BackgroundWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let cycle = (self.tick % 360) as f64;

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                // Base felt color with subtle variation
                let base_r = 22u8;
                let base_g = 22u8;
                let base_b = 52u8;

                // Subtle wave pattern
                let wave = ((x as f64 * 0.3 + y as f64 * 0.5 + cycle * 0.02).sin() * 4.0) as i16;

                let r = (base_r as i16 + wave).clamp(0, 255) as u8;
                let g = (base_g as i16 + wave / 2).clamp(0, 255) as u8;
                let b = (base_b as i16 + wave).clamp(0, 255) as u8;

                // Scanline effect: slightly dim every other row
                let (r, g, b) = if y % 2 == 0 {
                    (r, g, b)
                } else {
                    (
                        r.saturating_sub(3),
                        g.saturating_sub(3),
                        b.saturating_sub(3),
                    )
                };

                let cell = buf.cell_mut((x, y));
                if let Some(cell) = cell {
                    cell.set_symbol(" ");
                    cell.set_bg(Color::Rgb(r, g, b));
                }
            }
        }
    }
}

/// Render a decorative border/frame around the game area
pub struct FrameWidget {
    pub tick: u64,
}

impl FrameWidget {
    pub fn new(tick: u64) -> Self {
        Self { tick }
    }
}

impl Widget for FrameWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height < 4 {
            return;
        }

        let cycle = (self.tick % 600) as f64;

        // Top and bottom decorative lines
        for x in area.left()..area.right() {
            let t = (x as f64 / area.width as f64 + cycle * 0.005).sin().abs();
            let color = interpolate_color(Theme::FELT, Theme::GOLD, (t * 0.3) as f32);

            let top_cell = buf.cell_mut((x, area.top()));
            if let Some(cell) = top_cell {
                cell.set_symbol("\u{2500}"); // ─
                cell.set_fg(color);
            }

            let bot_cell = buf.cell_mut((x, area.bottom().saturating_sub(1)));
            if let Some(cell) = bot_cell {
                cell.set_symbol("\u{2500}");
                cell.set_fg(color);
            }
        }

        // Left and right borders
        for y in area.top()..area.bottom() {
            let t = (y as f64 / area.height as f64 + cycle * 0.005).sin().abs();
            let color = interpolate_color(Theme::FELT, Theme::GOLD, (t * 0.3) as f32);

            let left_cell = buf.cell_mut((area.left(), y));
            if let Some(cell) = left_cell {
                cell.set_symbol("\u{2502}"); // │
                cell.set_fg(color);
            }

            let right_cell = buf.cell_mut((area.right().saturating_sub(1), y));
            if let Some(cell) = right_cell {
                cell.set_symbol("\u{2502}");
                cell.set_fg(color);
            }
        }

        // Corners with gold accent
        let corner_style = Style::default().fg(Theme::GOLD);
        set_cell(buf, area.left(), area.top(), "\u{256d}", corner_style);
        set_cell(
            buf,
            area.right().saturating_sub(1),
            area.top(),
            "\u{256e}",
            corner_style,
        );
        set_cell(
            buf,
            area.left(),
            area.bottom().saturating_sub(1),
            "\u{2570}",
            corner_style,
        );
        set_cell(
            buf,
            area.right().saturating_sub(1),
            area.bottom().saturating_sub(1),
            "\u{256f}",
            corner_style,
        );
    }
}

fn set_cell(buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    let cell = buf.cell_mut((x, y));
    if let Some(cell) = cell {
        cell.set_symbol(symbol);
        if let Some(fg) = style.fg {
            cell.set_fg(fg);
        }
    }
}

fn interpolate_color(from: Color, to: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    match (from, to) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let r = (r1 as f32 + (r2 as f32 - r1 as f32) * t) as u8;
            let g = (g1 as f32 + (g2 as f32 - g1 as f32) * t) as u8;
            let b = (b1 as f32 + (b2 as f32 - b1 as f32) * t) as u8;
            Color::Rgb(r, g, b)
        }
        _ => to,
    }
}
