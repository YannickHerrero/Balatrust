use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};
use ratatui::Frame;

use balatrust_core::blind::BlindType;
use balatrust_core::consumable::ConsumableType;
use balatrust_core::joker::JokerRarity;
use balatrust_core::shop::ShopItem;
use balatrust_core::RunState;
use balatrust_widgets::consumable_slots::ConsumableSlotsWidget;
use balatrust_widgets::joker_bar::JokerBarWidget;
use balatrust_widgets::shop_panel::ShopPanelWidget;
use balatrust_widgets::sidebar::SidebarWidget;
use balatrust_widgets::theme::Theme;

use crate::app::ScreenAction;
use crate::screens::Screen;

// ─── Constants ────────────────────────────────────────────────────────

/// Width of the left sidebar in columns
const SIDEBAR_WIDTH: u16 = 30;
/// Width of the right sidebar (consumable slots) in columns
const RIGHT_SIDEBAR_WIDTH: u16 = 14;

/// Focus area in the shop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShopFocus {
    Items,
    Jokers,
}

pub struct ShopScreen {
    pub cursor: usize,
    pub focus: ShopFocus,
    pub joker_cursor: usize,

    // Cached rects for mouse hit-testing
    joker_rects: Vec<Rect>,
    item_rects: Vec<Rect>,
    next_round_rect: Rect,
    reroll_rect: Rect,
    shop_panel_rect: Rect,

    // Popup state
    inspected_item: Option<usize>,
    inspected_joker: Option<usize>,
}

impl ShopScreen {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            focus: ShopFocus::Items,
            joker_cursor: 0,
            joker_rects: Vec::new(),
            item_rects: Vec::new(),
            next_round_rect: Rect::default(),
            reroll_rect: Rect::default(),
            shop_panel_rect: Rect::default(),
            inspected_item: None,
            inspected_joker: None,
        }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
        self.focus = ShopFocus::Items;
        self.joker_cursor = 0;
        self.joker_rects.clear();
        self.item_rects.clear();
        self.next_round_rect = Rect::default();
        self.reroll_rect = Rect::default();
        self.shop_panel_rect = Rect::default();
        self.inspected_item = None;
        self.inspected_joker = None;
    }

    // ─── Sidebar Data ─────────────────────────────────────────────────

    fn sidebar_data(&self, game: &RunState) -> SidebarWidget {
        let blind_color = match game.blind_type {
            BlindType::Small => Theme::SMALL_BLIND,
            BlindType::Big => Theme::BIG_BLIND,
            BlindType::Boss(_) => Theme::BOSS_BLIND,
        };

        SidebarWidget::new(
            game.blind_type.name(),
            blind_color,
            game.score_target,
            game.blind_type.reward(),
            game.round_score,
            String::new(), // No hand type in shop
            1,
            0,
            0,
            game.hands_remaining,
            game.discards_remaining,
            game.money,
            game.ante,
            8, // max_ante
            game.round_number(),
        )
        .shop(true)
    }

    // ─── Rendering ────────────────────────────────────────────────────

    pub fn render(&mut self, frame: &mut Frame, game: &Option<RunState>) {
        let area = frame.area();

        let game = match game {
            Some(g) => g,
            None => return,
        };

        // ═══════════════════════════════════════════════════════════════
        // 3-COLUMN LAYOUT (same as play_round)
        // ═══════════════════════════════════════════════════════════════

        let columns = Layout::horizontal([
            Constraint::Length(SIDEBAR_WIDTH),       // Left sidebar
            Constraint::Min(40),                     // Center area
            Constraint::Length(RIGHT_SIDEBAR_WIDTH), // Right sidebar
        ])
        .split(area);

        // ═══ LEFT SIDEBAR (shop mode) ═══
        let sidebar = self.sidebar_data(game);
        frame.render_widget(sidebar, columns[0]);

        // ═══ RIGHT SIDEBAR (consumable slots) ═══
        frame.render_widget(
            ConsumableSlotsWidget::new(&game.consumables, game.max_consumables),
            columns[2],
        );

        // ═══ CENTER AREA ═══
        let center = columns[1];
        self.render_center(frame, game, center);

        // ═══ OVERLAYS ═══

        // Item inspect popup
        if let Some(idx) = self.inspected_item {
            self.render_item_inspect(frame, game, idx, area);
        }

        // Joker inspect popup
        if let Some(ji) = self.inspected_joker {
            self.render_joker_inspect(frame, game, ji, area);
        }
    }

    fn render_center(&mut self, frame: &mut Frame, game: &RunState, center: Rect) {
        // ┌─────────────────────────────────────┐
        // │ Joker bar (6)                        │
        // │ Shop panel (flex)                    │
        // │ Help line (1)                        │
        // └─────────────────────────────────────┘

        let rows = Layout::vertical([
            Constraint::Length(6), // Joker bar + counter
            Constraint::Min(0),    // Shop panel
            Constraint::Length(1), // Help line
        ])
        .split(center);

        // === Joker bar ===
        self.render_joker_bar(frame, game, rows[0]);

        // === Shop panel ===
        self.shop_panel_rect = rows[1];
        let items = game
            .shop
            .as_ref()
            .map(|s| s.items.as_slice())
            .unwrap_or(&[]);
        let reroll_cost = game.shop.as_ref().map_or(5, |s| s.reroll_cost);

        let selected_item = if self.focus == ShopFocus::Items {
            Some(self.cursor)
        } else {
            None
        };

        let panel = ShopPanelWidget::new(items, game.money, reroll_cost, selected_item);

        // Cache hit-test rects
        self.next_round_rect = ShopPanelWidget::next_round_rect(rows[1]);
        self.reroll_rect = ShopPanelWidget::reroll_rect(rows[1]);
        self.item_rects = ShopPanelWidget::item_rects(rows[1], items.len());

        frame.render_widget(panel, rows[1]);

        // === Help line ===
        let reroll_cost_str = format!("${}", reroll_cost);
        let help = Paragraph::new(Line::from(vec![
            Span::styled("[", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("N", Style::default().fg(Theme::GOLD)),
            Span::styled("] Next Round  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("R", Style::default().fg(Theme::GOLD)),
            Span::styled(
                format!("] Reroll ({})  [", reroll_cost_str),
                Style::default().fg(Theme::DIM_TEXT),
            ),
            Span::styled("S", Style::default().fg(Theme::GOLD)),
            Span::styled("] Sell Joker  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("Tab", Style::default().fg(Theme::GOLD)),
            Span::styled("] Switch  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("\u{2190}\u{2192}", Style::default().fg(Theme::GOLD)),
            Span::styled("] Move  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("Enter", Style::default().fg(Theme::GOLD)),
            Span::styled("] Select", Style::default().fg(Theme::DIM_TEXT)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(help, rows[2]);
    }

    fn render_joker_bar(&mut self, frame: &mut Frame, game: &RunState, area: Rect) {
        let parts = Layout::vertical([
            Constraint::Length(5), // Joker cards
            Constraint::Length(1), // Counter
        ])
        .split(area);

        let joker_bar = JokerBarWidget::new(&game.jokers, game.max_jokers).selected(
            if self.focus == ShopFocus::Jokers {
                Some(self.joker_cursor)
            } else {
                None
            },
        );

        // Cache joker rects
        self.joker_rects.clear();
        for i in 0..game.jokers.len() {
            if let Some(rect) = joker_bar.joker_rect(parts[0], i) {
                self.joker_rects.push(rect);
            } else {
                self.joker_rects.push(Rect::default());
            }
        }
        frame.render_widget(joker_bar, parts[0]);

        // Joker slot counter
        let counter = format!("{}/{}", game.jokers.len(), game.max_jokers);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                counter,
                Style::default().fg(Theme::MUTED_TEXT),
            )))
            .alignment(Alignment::Center),
            parts[1],
        );
    }

    // ─── Popup Overlays ───────────────────────────────────────────────

    fn render_item_inspect(
        &self,
        frame: &mut Frame,
        game: &RunState,
        item_index: usize,
        _screen_area: Rect,
    ) {
        let items = match &game.shop {
            Some(shop) => &shop.items,
            None => return,
        };

        let item = match items.get(item_index) {
            Some(i) => i,
            None => return,
        };

        let item_rect = match self.item_rects.get(item_index).copied() {
            Some(r) if r.width > 0 => r,
            _ => return,
        };

        let name = item.name();
        let desc = item.description();
        let price = item.price();
        let can_afford = game.money >= price;

        let (type_label, name_color) = match item {
            ShopItem::JokerItem(j) => {
                let color = match j.joker_type.rarity() {
                    JokerRarity::Common => Theme::COMMON,
                    JokerRarity::Uncommon => Theme::UNCOMMON,
                    JokerRarity::Rare => Theme::RARE,
                    JokerRarity::Legendary => Theme::LEGENDARY,
                };
                ("Joker", color)
            }
            ShopItem::ConsumableItem(c) => match c.consumable_type {
                ConsumableType::Planet(_) => ("Planet", Theme::CHIPS_COLOR),
                ConsumableType::Tarot(_) => ("Tarot", Theme::LEGENDARY),
            },
        };

        let mut lines: Vec<Line> = vec![
            Line::from(Span::styled(
                name,
                Style::default().fg(name_color).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(desc, Style::default().fg(Theme::MUTED_TEXT))),
            Line::from(""),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Theme::DIM_TEXT)),
                Span::styled(
                    type_label,
                    Style::default()
                        .fg(Theme::BRIGHT_TEXT)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Price: ", Style::default().fg(Theme::DIM_TEXT)),
                Span::styled(
                    format!("${}", price),
                    Style::default()
                        .fg(if can_afford {
                            Theme::MONEY_COLOR
                        } else {
                            Theme::MULT_COLOR
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
        ];

        // Buy button line
        if can_afford {
            lines.push(Line::from(Span::styled(
                format!("   Buy ${}   ", price),
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "  Can't afford  ",
                Style::default().fg(Theme::DIM_TEXT),
            )));
        }

        let content_width = lines.iter().map(|l| l.width() as u16).max().unwrap_or(10) + 4;
        let popup_width = content_width.clamp(24, 40);
        let popup_height = (lines.len() as u16) + 3;

        let popup_x = item_rect
            .x
            .saturating_add(item_rect.width / 2)
            .saturating_sub(popup_width / 2)
            .max(frame.area().x)
            .min(frame.area().right().saturating_sub(popup_width));
        let popup_y = item_rect
            .bottom()
            .min(frame.area().bottom().saturating_sub(popup_height));

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        frame.render_widget(ratatui::widgets::Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(name_color))
            .padding(Padding::horizontal(1));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        for (i, line) in lines.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            frame.render_widget(
                Paragraph::new(line.clone()),
                Rect::new(inner.x, y, inner.width, 1),
            );
        }
    }

    fn render_joker_inspect(
        &self,
        frame: &mut Frame,
        game: &RunState,
        joker_index: usize,
        _screen_area: Rect,
    ) {
        let joker = match game.jokers.get(joker_index) {
            Some(j) => j,
            None => return,
        };

        let joker_rect = match self.joker_rects.get(joker_index).copied() {
            Some(r) if r.width > 0 => r,
            _ => return,
        };

        let jt = &joker.joker_type;
        let name = jt.name();
        let desc = jt.description();
        let rarity = jt.rarity();
        let sell_value = joker.total_sell_value();

        let rarity_str = match rarity {
            JokerRarity::Common => "Common",
            JokerRarity::Uncommon => "Uncommon",
            JokerRarity::Rare => "Rare",
            JokerRarity::Legendary => "Legendary",
        };
        let rarity_color = match rarity {
            JokerRarity::Common => Theme::COMMON,
            JokerRarity::Uncommon => Theme::UNCOMMON,
            JokerRarity::Rare => Theme::RARE,
            JokerRarity::Legendary => Theme::LEGENDARY,
        };

        let lines: Vec<Line> = vec![
            Line::from(Span::styled(
                name,
                Style::default()
                    .fg(Theme::BRIGHT_TEXT)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(desc, Style::default().fg(Theme::MUTED_TEXT))),
            Line::from(""),
            Line::from(vec![
                Span::styled("Rarity: ", Style::default().fg(Theme::DIM_TEXT)),
                Span::styled(
                    rarity_str,
                    Style::default()
                        .fg(rarity_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Sell: ", Style::default().fg(Theme::DIM_TEXT)),
                Span::styled(
                    format!("${}", sell_value),
                    Style::default()
                        .fg(Theme::MONEY_COLOR)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let content_width = lines.iter().map(|l| l.width() as u16).max().unwrap_or(10) + 4;
        let popup_width = content_width.clamp(20, 40);
        let popup_height = (lines.len() as u16) + 3;

        let popup_x = joker_rect
            .x
            .saturating_add(joker_rect.width / 2)
            .saturating_sub(popup_width / 2)
            .max(frame.area().x)
            .min(frame.area().right().saturating_sub(popup_width));
        let popup_y = joker_rect
            .bottom()
            .min(frame.area().bottom().saturating_sub(popup_height));

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        frame.render_widget(ratatui::widgets::Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(rarity_color))
            .padding(Padding::horizontal(1));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        for (i, line) in lines.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            frame.render_widget(
                Paragraph::new(line.clone()),
                Rect::new(inner.x, y, inner.width, 1),
            );
        }
    }

    // ─── Buy Button Hit Test ──────────────────────────────────────────

    /// Check if a click hits the "Buy" button in the item inspect popup.
    /// The buy button is the last line content in the popup, positioned below the item card.
    fn hit_test_buy_button(&self, game: &RunState, col: u16, row: u16) -> bool {
        let item_index = match self.inspected_item {
            Some(idx) => idx,
            None => return false,
        };

        let items = match &game.shop {
            Some(shop) => &shop.items,
            None => return false,
        };

        let item = match items.get(item_index) {
            Some(i) => i,
            None => return false,
        };

        if game.money < item.price() {
            return false;
        }

        let item_rect = match self.item_rects.get(item_index).copied() {
            Some(r) if r.width > 0 => r,
            _ => return false,
        };

        // Reconstruct the popup area to check if click is on the buy button region
        // The popup has ~11 lines of content + 2 border = ~13 height
        let popup_height = 11u16 + 3;
        let popup_width = 28u16; // approximate

        let popup_x = item_rect
            .x
            .saturating_add(item_rect.width / 2)
            .saturating_sub(popup_width / 2);
        let popup_y = item_rect.bottom();

        // Buy button is roughly at the bottom of the popup content
        // Inside the popup: border(1) + 7 content lines + buy button line = row at y+9
        let buy_y = popup_y + popup_height - 3; // approximate row of "Buy" button

        // Check if click is roughly in the popup area on the buy button row
        col >= popup_x
            && col < popup_x + popup_width
            && row >= buy_y.saturating_sub(1)
            && row <= buy_y + 1
    }

    // ─── Input Handling ───────────────────────────────────────────────

    pub fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
        game: &Option<RunState>,
    ) -> Option<ScreenAction> {
        let game = match game {
            Some(g) => g,
            None => return None,
        };

        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            let col = mouse.column;
            let row = mouse.row;

            // If item inspect popup is open, check buy button first
            if let Some(idx) = self.inspected_item {
                if self.hit_test_buy_button(game, col, row) {
                    self.inspected_item = None;
                    return Some(ScreenAction::BuyShopItem(idx));
                }
                // Click anywhere else dismisses the popup
                self.inspected_item = None;
                return None;
            }

            // If joker inspect popup is open, dismiss on any click
            if self.inspected_joker.is_some() {
                self.inspected_joker = None;
                return None;
            }

            // Check "Next Round" button
            if self.next_round_rect.width > 0
                && col >= self.next_round_rect.x
                && col < self.next_round_rect.x + self.next_round_rect.width
                && row >= self.next_round_rect.y
                && row < self.next_round_rect.y + self.next_round_rect.height
            {
                return Some(ScreenAction::LeaveShop);
            }

            // Check "Reroll" button
            if self.reroll_rect.width > 0
                && col >= self.reroll_rect.x
                && col < self.reroll_rect.x + self.reroll_rect.width
                && row >= self.reroll_rect.y
                && row < self.reroll_rect.y + self.reroll_rect.height
            {
                return Some(ScreenAction::RerollShop);
            }

            // Check shop item cards (open inspect popup)
            for (i, rect) in self.item_rects.iter().enumerate() {
                if rect.width > 0
                    && col >= rect.x
                    && col < rect.x + rect.width
                    && row >= rect.y
                    && row < rect.y + rect.height
                {
                    self.inspected_item = Some(i);
                    self.focus = ShopFocus::Items;
                    self.cursor = i;
                    return None;
                }
            }

            // Check jokers (toggle inspect popup)
            for (i, rect) in self.joker_rects.iter().enumerate() {
                if rect.width > 0
                    && col >= rect.x
                    && col < rect.x + rect.width
                    && row >= rect.y
                    && row < rect.y + rect.height
                {
                    if self.inspected_joker == Some(i) {
                        self.inspected_joker = None;
                    } else {
                        self.inspected_joker = Some(i);
                    }
                    return None;
                }
            }
        }
        None
    }
}

impl Screen for ShopScreen {
    fn render(&mut self, frame: &mut Frame, game: &Option<RunState>) {
        // Delegate to the inherent method
        ShopScreen::render(self, frame, game);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenAction> {
        // Dismiss popups first
        if let Some(idx) = self.inspected_item {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                self.inspected_item = None;
                return None;
            }
            if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
                // Buy the inspected item
                self.inspected_item = None;
                return Some(ScreenAction::BuyShopItem(idx));
            }
            self.inspected_item = None;
            return None;
        }

        if self.inspected_joker.is_some() {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ')) {
                self.inspected_joker = None;
            }
            return None;
        }

        match key.code {
            KeyCode::Char('n') | KeyCode::Char('N') => {
                return Some(ScreenAction::LeaveShop);
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    ShopFocus::Items => ShopFocus::Jokers,
                    ShopFocus::Jokers => ShopFocus::Items,
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
            },
            KeyCode::Right | KeyCode::Char('l') => match self.focus {
                ShopFocus::Items => self.cursor += 1,
                ShopFocus::Jokers => self.joker_cursor += 1,
            },
            KeyCode::Enter | KeyCode::Char(' ') => match self.focus {
                ShopFocus::Items => {
                    // Open the inspect popup for the current item
                    self.inspected_item = Some(self.cursor);
                }
                ShopFocus::Jokers => {
                    // Open joker inspect
                    self.inspected_joker = Some(self.joker_cursor);
                }
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
