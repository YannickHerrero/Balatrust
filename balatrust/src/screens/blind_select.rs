use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use balatrust_core::blind::BlindType;
use balatrust_core::run::BlindOutcome;
use balatrust_core::RunState;
use balatrust_widgets::blind_select::BlindSelectWidget;
use balatrust_widgets::consumable_slots::ConsumableSlotsWidget;
use balatrust_widgets::joker_bar::JokerBarWidget;
use balatrust_widgets::sidebar::SidebarWidget;
use balatrust_widgets::theme::Theme;

use crate::app::ScreenAction;
use crate::screens::Screen;

// ─── Constants ────────────────────────────────────────────────────────

const SIDEBAR_WIDTH: u16 = 30;
const RIGHT_SIDEBAR_WIDTH: u16 = 14;

pub struct BlindSelectScreen {
    pub cursor: usize, // 0=small, 1=big, 2=boss

    // Cached rects for mouse hit-testing
    joker_rects: Vec<Rect>,
    card_rects: [Rect; 3],
    select_button_rects: [Rect; 3],
    skip_button_rects: [Rect; 3],
    panel_rect: Rect,

    // Popup state
    inspected_joker: Option<usize>,
}

impl BlindSelectScreen {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            joker_rects: Vec::new(),
            card_rects: [Rect::default(); 3],
            select_button_rects: [Rect::default(); 3],
            skip_button_rects: [Rect::default(); 3],
            panel_rect: Rect::default(),
            inspected_joker: None,
        }
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
            String::new(), // No hand type on blind select
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
        .blind_select(true)
    }

    // ─── Rendering ────────────────────────────────────────────────────

    pub fn render(&mut self, frame: &mut Frame, game: &Option<RunState>) {
        let area = frame.area();

        let game = match game {
            Some(g) => g,
            None => return,
        };

        // 3-column layout
        let columns = Layout::horizontal([
            Constraint::Length(SIDEBAR_WIDTH),
            Constraint::Min(40),
            Constraint::Length(RIGHT_SIDEBAR_WIDTH),
        ])
        .split(area);

        // ═══ LEFT SIDEBAR ═══
        let sidebar = self.sidebar_data(game);
        frame.render_widget(sidebar, columns[0]);

        // ═══ RIGHT SIDEBAR ═══
        frame.render_widget(
            ConsumableSlotsWidget::new(&game.consumables, game.max_consumables),
            columns[2],
        );

        // ═══ CENTER AREA ═══
        let center = columns[1];
        self.render_center(frame, game, center);

        // ═══ OVERLAYS ═══
        if let Some(ji) = self.inspected_joker {
            self.render_joker_inspect(frame, game, ji, area);
        }
    }

    fn render_center(&mut self, frame: &mut Frame, game: &RunState, center: Rect) {
        let rows = Layout::vertical([
            Constraint::Length(6), // Joker bar + counter
            Constraint::Min(0),    // Blind cards panel
            Constraint::Length(1), // Help line
        ])
        .split(center);

        // === Joker bar ===
        self.render_joker_bar(frame, game, rows[0]);

        // === Blind cards panel ===
        self.panel_rect = rows[1];

        // Clamp cursor to active blind
        let active_index = game.current_blind_index();

        let widget =
            BlindSelectWidget::new(game.ante, game.boss_blind, self.cursor, game.blind_outcomes);

        // Cache hit-test rects
        for i in 0..3 {
            self.card_rects[i] = BlindSelectWidget::card_rect(rows[1], i);
            self.select_button_rects[i] = BlindSelectWidget::select_button_rect(rows[1], i);
            self.skip_button_rects[i] = BlindSelectWidget::skip_button_rect(rows[1], i);
        }

        frame.render_widget(widget, rows[1]);

        // === Help line ===
        let can_skip = active_index < 2; // Can skip Small/Big only
        let mut help_spans = vec![
            Span::styled("[", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("\u{2190}\u{2192}", Style::default().fg(Theme::GOLD)),
            Span::styled("] Move  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("Enter", Style::default().fg(Theme::GOLD)),
            Span::styled("] Select", Style::default().fg(Theme::DIM_TEXT)),
        ];
        if can_skip {
            help_spans.push(Span::styled("  [", Style::default().fg(Theme::DIM_TEXT)));
            help_spans.push(Span::styled("S", Style::default().fg(Theme::GOLD)));
            help_spans.push(Span::styled("] Skip", Style::default().fg(Theme::DIM_TEXT)));
        }
        let help = Paragraph::new(Line::from(help_spans)).alignment(Alignment::Center);
        frame.render_widget(help, rows[2]);
    }

    fn render_joker_bar(&mut self, frame: &mut Frame, game: &RunState, area: Rect) {
        let parts = Layout::vertical([Constraint::Length(5), Constraint::Length(1)]).split(area);

        let joker_bar = JokerBarWidget::new(&game.jokers, game.max_jokers);

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

    // ─── Joker Inspect Popup ──────────────────────────────────────────

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
            balatrust_core::joker::JokerRarity::Common => "Common",
            balatrust_core::joker::JokerRarity::Uncommon => "Uncommon",
            balatrust_core::joker::JokerRarity::Rare => "Rare",
            balatrust_core::joker::JokerRarity::Legendary => "Legendary",
        };
        let rarity_color = match rarity {
            balatrust_core::joker::JokerRarity::Common => Theme::COMMON,
            balatrust_core::joker::JokerRarity::Uncommon => Theme::UNCOMMON,
            balatrust_core::joker::JokerRarity::Rare => Theme::RARE,
            balatrust_core::joker::JokerRarity::Legendary => Theme::LEGENDARY,
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

        let block = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(rarity_color))
            .padding(ratatui::widgets::Padding::horizontal(1));

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

    // ─── Mouse Handling ───────────────────────────────────────────────

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

            // Dismiss joker popup first
            if self.inspected_joker.is_some() {
                self.inspected_joker = None;
                return None;
            }

            let active_index = game.current_blind_index();

            // Check "Select" button on the active blind card
            let select_rect = self.select_button_rects[active_index];
            if select_rect.width > 0
                && col >= select_rect.x
                && col < select_rect.x + select_rect.width
                && row >= select_rect.y
                && row < select_rect.y + select_rect.height
            {
                self.cursor = active_index;
                return Some(ScreenAction::StartBlind);
            }

            // Check "Skip" button on active blind card (only Small/Big)
            if active_index < 2 {
                let skip_rect = self.skip_button_rects[active_index];
                if skip_rect.width > 0
                    && col >= skip_rect.x
                    && col < skip_rect.x + skip_rect.width
                    && row >= skip_rect.y
                    && row < skip_rect.y + skip_rect.height
                {
                    self.cursor = active_index;
                    return Some(ScreenAction::SkipBlind);
                }
            }

            // Check if clicking on a blind card (to move cursor)
            for i in 0..3 {
                let rect = self.card_rects[i];
                if rect.width > 0
                    && col >= rect.x
                    && col < rect.x + rect.width
                    && row >= rect.y
                    && row < rect.y + rect.height
                {
                    // Only allow cursor on active and past blinds, not upcoming
                    if game.blind_outcomes[i] != BlindOutcome::Upcoming {
                        self.cursor = i;
                    }
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

impl Screen for BlindSelectScreen {
    fn render(&mut self, frame: &mut Frame, game: &Option<RunState>) {
        BlindSelectScreen::render(self, frame, game);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenAction> {
        // Dismiss joker popup first
        if self.inspected_joker.is_some() {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ')) {
                self.inspected_joker = None;
            }
            return None;
        }

        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.cursor < 2 {
                    self.cursor += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                return Some(ScreenAction::StartBlind);
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                return Some(ScreenAction::SkipBlind);
            }
            _ => {}
        }
        None
    }
}
