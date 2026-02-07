use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};
use ratatui::Frame;

use balatrust_core::joker::JokerRarity;
use balatrust_core::shop::ShopItem;
use balatrust_core::RunState;
use balatrust_widgets::joker_bar::JokerBarWidget;
use balatrust_widgets::theme::Theme;

use crate::app::ScreenAction;
use crate::screens::Screen;

/// Focus area in the shop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShopFocus {
    Items,
    Jokers,
    Consumables,
}

pub struct ShopScreen {
    pub cursor: usize,
    pub focus: ShopFocus,
    pub joker_cursor: usize,
    pub consumable_cursor: usize,
}

impl ShopScreen {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            focus: ShopFocus::Items,
            joker_cursor: 0,
            consumable_cursor: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
        self.focus = ShopFocus::Items;
        self.joker_cursor = 0;
        self.consumable_cursor = 0;
    }
}

impl Screen for ShopScreen {
    fn render(&mut self, frame: &mut Frame, game: &Option<RunState>) {
        let area = frame.area();

        let game = match game {
            Some(g) => g,
            None => return,
        };

        let chunks = Layout::vertical([
            Constraint::Length(3), // Header
            Constraint::Length(5), // Joker bar (owned)
            Constraint::Length(1), // Separator
            Constraint::Min(0),    // Shop items
            Constraint::Length(3), // Consumables
            Constraint::Length(3), // Footer
        ])
        .split(area);

        // Header
        let reward_info = format!("Interest: +${}", (game.money / 5).min(5));
        let header = Line::from(vec![
            Span::styled(
                format!("  ${}  ", game.money),
                Style::default()
                    .fg(Theme::MONEY_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("\u{2502} ", Style::default().fg(Theme::CARD_BORDER)),
            Span::styled(
                "SHOP",
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" \u{2502} Ante {} / {}", game.ante, game.blind_type),
                Style::default().fg(Theme::MUTED_TEXT),
            ),
            Span::styled(
                format!("  \u{2502} {}", reward_info),
                Style::default().fg(Theme::MONEY_COLOR),
            ),
        ]);
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Theme::CARD_BORDER));
        let header_inner = header_block.inner(chunks[0]);
        frame.render_widget(header_block, chunks[0]);
        frame.render_widget(Paragraph::new(header), header_inner);

        // Joker bar
        let joker_bar = JokerBarWidget::new(&game.jokers, game.max_jokers).selected(
            if self.focus == ShopFocus::Jokers {
                Some(self.joker_cursor)
            } else {
                None
            },
        );
        frame.render_widget(joker_bar, chunks[1]);

        // Shop items
        self.render_shop_items(frame, game, chunks[3]);

        // Consumables
        self.render_consumables(frame, game, chunks[4]);

        // Footer
        let reroll_cost = game.shop.as_ref().map_or(5, |s| s.reroll_cost);
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("[", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("Enter", Style::default().fg(Theme::GOLD)),
            Span::styled("] Buy  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("R", Style::default().fg(Theme::GOLD)),
            Span::styled(
                format!("] Reroll (${})", reroll_cost),
                Style::default().fg(Theme::DIM_TEXT),
            ),
            Span::styled("  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("S", Style::default().fg(Theme::GOLD)),
            Span::styled("] Sell Joker  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("Tab", Style::default().fg(Theme::GOLD)),
            Span::styled("] Switch Focus  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("N", Style::default().fg(Theme::GOLD)),
            Span::styled("] Next Round", Style::default().fg(Theme::DIM_TEXT)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[5]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenAction> {
        match key.code {
            KeyCode::Char('n') | KeyCode::Char('N') => {
                return Some(ScreenAction::LeaveShop);
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    ShopFocus::Items => ShopFocus::Jokers,
                    ShopFocus::Jokers => ShopFocus::Consumables,
                    ShopFocus::Consumables => ShopFocus::Items,
                };
            }
            KeyCode::Left | KeyCode::Char('h') => match self.focus {
                ShopFocus::Items => {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                    }
                }
                ShopFocus::Jokers => {
                    if self.joker_cursor > 0 {
                        self.joker_cursor -= 1;
                    }
                }
                ShopFocus::Consumables => {
                    if self.consumable_cursor > 0 {
                        self.consumable_cursor -= 1;
                    }
                }
            },
            KeyCode::Right | KeyCode::Char('l') => match self.focus {
                ShopFocus::Items => self.cursor += 1,
                ShopFocus::Jokers => self.joker_cursor += 1,
                ShopFocus::Consumables => self.consumable_cursor += 1,
            },
            KeyCode::Enter => match self.focus {
                ShopFocus::Items => {
                    return Some(ScreenAction::BuyShopItem(self.cursor));
                }
                ShopFocus::Consumables => {
                    return Some(ScreenAction::UseConsumable(self.consumable_cursor));
                }
                _ => {}
            },
            KeyCode::Char('r') | KeyCode::Char('R') => {
                return Some(ScreenAction::RerollShop);
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                if self.focus == ShopFocus::Jokers {
                    return Some(ScreenAction::SellJoker(self.joker_cursor));
                }
            }
            _ => {}
        }
        None
    }
}

impl ShopScreen {
    fn render_shop_items(&self, frame: &mut Frame, game: &RunState, area: ratatui::layout::Rect) {
        let items = match &game.shop {
            Some(shop) => &shop.items,
            None => return,
        };

        if items.is_empty() {
            let empty = Paragraph::new(Line::from(Span::styled(
                "No items available - [R]eroll or [N]ext Round",
                Style::default().fg(Theme::DIM_TEXT),
            )))
            .alignment(Alignment::Center);
            frame.render_widget(empty, area);
            return;
        }

        let item_width = 22u16;
        let spacing = 2u16;
        let total_w = items.len() as u16 * item_width + (items.len() as u16 - 1) * spacing;
        let start_x = area.x + area.width.saturating_sub(total_w) / 2;

        for (i, item) in items.iter().enumerate() {
            let x = start_x + i as u16 * (item_width + spacing);
            let item_area = ratatui::layout::Rect::new(x, area.y, item_width, area.height);

            if item_area.right() > area.right() {
                break;
            }

            let is_selected = self.focus == ShopFocus::Items && self.cursor == i;
            let can_afford = game.money >= item.price();

            self.render_shop_item(frame, item, item_area, is_selected, can_afford);
        }
    }

    fn render_shop_item(
        &self,
        frame: &mut Frame,
        item: &ShopItem,
        area: ratatui::layout::Rect,
        selected: bool,
        can_afford: bool,
    ) {
        let border_color = if selected {
            Theme::CARD_SELECTED
        } else {
            Theme::CARD_BORDER
        };

        let border_type = if selected {
            BorderType::Double
        } else {
            BorderType::Rounded
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color))
            .padding(Padding::horizontal(1));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height < 3 || inner.width < 8 {
            return;
        }

        let mut y = inner.y;

        // Item name
        let name = item.name();
        let max_len = inner.width as usize;
        let display_name: String = name.chars().take(max_len).collect();

        let name_color = match item {
            ShopItem::JokerItem(j) => match j.joker_type.rarity() {
                JokerRarity::Common => Theme::COMMON,
                JokerRarity::Uncommon => Theme::UNCOMMON,
                JokerRarity::Rare => Theme::RARE,
                JokerRarity::Legendary => Theme::LEGENDARY,
            },
            ShopItem::ConsumableItem(_) => Theme::CHIPS_COLOR,
        };

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                display_name,
                Style::default().fg(name_color).add_modifier(Modifier::BOLD),
            ))),
            ratatui::layout::Rect::new(inner.x, y, inner.width, 1),
        );
        y += 1;

        // Description
        if y < inner.bottom() {
            let desc = item.description();
            let display_desc: String = desc.chars().take(max_len).collect();
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    display_desc,
                    Style::default().fg(Theme::MUTED_TEXT),
                ))),
                ratatui::layout::Rect::new(inner.x, y, inner.width, 1),
            );
            y += 1;
        }

        // Price
        if y < inner.bottom() {
            let price_color = if can_afford {
                Theme::MONEY_COLOR
            } else {
                Theme::MULT_COLOR
            };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("${}", item.price()),
                    Style::default()
                        .fg(price_color)
                        .add_modifier(Modifier::BOLD),
                ))),
                ratatui::layout::Rect::new(inner.x, y, inner.width, 1),
            );
        }
    }

    fn render_consumables(&self, frame: &mut Frame, game: &RunState, area: ratatui::layout::Rect) {
        if game.consumables.is_empty() {
            let label = Paragraph::new(Line::from(Span::styled(
                "Consumables: (empty)",
                Style::default().fg(Theme::DIM_TEXT),
            )))
            .alignment(Alignment::Center);
            frame.render_widget(label, area);
            return;
        }

        let mut spans = vec![Span::styled(
            "Consumables: ",
            Style::default().fg(Theme::MUTED_TEXT),
        )];

        for (i, cons) in game.consumables.iter().enumerate() {
            let is_sel = self.focus == ShopFocus::Consumables && self.consumable_cursor == i;
            let style = if is_sel {
                Style::default()
                    .fg(Theme::CARD_SELECTED)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::CHIPS_COLOR)
            };
            let prefix = if is_sel { ">" } else { " " };
            spans.push(Span::styled(
                format!("{}[{}] ", prefix, cons.consumable_type.name()),
                style,
            ));
        }

        let line = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(line, area);
    }
}
