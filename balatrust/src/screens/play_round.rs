use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use balatrust_core::hand::detect_hand;
use balatrust_core::scoring::ScoreResult;
use balatrust_core::{PlayingCard, RunState};
use balatrust_widgets::hand::HandWidget;
use balatrust_widgets::hud::HudWidget;
use balatrust_widgets::joker_bar::JokerBarWidget;
use balatrust_widgets::score_display::ScoreDisplayWidget;
use balatrust_widgets::theme::Theme;

use crate::app::ScreenAction;

pub struct PlayRoundScreen {
    pub cursor: usize,
    pub last_score: Option<ScoreResult>,
    pub last_played: Vec<PlayingCard>,
    pub blind_just_beaten: bool,
    hand_card_rects: Vec<Rect>,
}

impl PlayRoundScreen {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            last_score: None,
            last_played: Vec::new(),
            blind_just_beaten: false,
            hand_card_rects: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
        self.last_score = None;
        self.last_played.clear();
        self.blind_just_beaten = false;
        self.hand_card_rects.clear();
    }

    pub fn render(&mut self, frame: &mut Frame, game: &Option<RunState>) {
        let area = frame.area();
        let bg = Block::default().style(Style::default().bg(Theme::BG));
        frame.render_widget(bg, area);

        let game = match game {
            Some(g) => g,
            None => return,
        };

        // Main layout: header | joker bar | last score | game area | footer
        let main_chunks = Layout::vertical([
            Constraint::Length(3), // Header (blind info)
            Constraint::Length(5), // Joker bar
            Constraint::Length(2), // Last played cards info
            Constraint::Min(0),    // Hand area
            Constraint::Length(1), // HUD
            Constraint::Length(2), // Help
        ])
        .split(area);

        // === Header: Blind info ===
        self.render_header(frame, game, main_chunks[0]);

        // === Joker bar ===
        let joker_bar = JokerBarWidget::new(&game.jokers, game.max_jokers);
        frame.render_widget(joker_bar, main_chunks[1]);

        // === Last score display ===
        self.render_last_score(frame, main_chunks[2]);

        // === Game area: score panel + hand ===
        let game_area = Layout::horizontal([
            Constraint::Min(0),     // Hand cards
            Constraint::Length(22), // Score panel
        ])
        .split(main_chunks[3]);

        // Render hand
        self.render_hand(frame, game, game_area[0]);

        // Render score panel
        self.render_score_panel(frame, game, game_area[1]);

        // === HUD ===
        let hud = HudWidget::new(
            game.hands_remaining,
            game.discards_remaining,
            game.money,
            game.deck.remaining(),
        )
        .can_play(game.can_play())
        .can_discard(game.can_discard());
        frame.render_widget(hud, main_chunks[4]);

        // === Help line ===
        let help = Paragraph::new(Line::from(vec![
            Span::styled("[", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("\u{2190}\u{2192}", Style::default().fg(Theme::GOLD)),
            Span::styled("] Move  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("Space", Style::default().fg(Theme::GOLD)),
            Span::styled("] Select  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("P", Style::default().fg(Theme::GOLD)),
            Span::styled("] Play  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("D", Style::default().fg(Theme::GOLD)),
            Span::styled("] Discard  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("A", Style::default().fg(Theme::GOLD)),
            Span::styled("] Select All  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("C", Style::default().fg(Theme::GOLD)),
            Span::styled("] Clear", Style::default().fg(Theme::DIM_TEXT)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(help, main_chunks[5]);

        // Blind beaten popup
        if self.blind_just_beaten {
            self.render_beaten_popup(frame, game, area);
        }
    }

    fn render_header(&self, frame: &mut Frame, game: &RunState, area: Rect) {
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Theme::CARD_BORDER));

        let inner = header_block.inner(area);
        frame.render_widget(header_block, area);

        let header = Line::from(vec![
            Span::styled(
                format!("  Ante {}  ", game.ante),
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("\u{2502} ", Style::default().fg(Theme::CARD_BORDER)),
            Span::styled(
                format!("{}", game.blind_type),
                Style::default()
                    .fg(Theme::BRIGHT_TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" \u{2502} ", Style::default().fg(Theme::CARD_BORDER)),
            Span::styled("Target: ", Style::default().fg(Theme::MUTED_TEXT)),
            Span::styled(
                format!("{}", game.score_target),
                Style::default()
                    .fg(Theme::CHIPS_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(header), inner);
    }

    fn render_last_score(&self, frame: &mut Frame, area: Rect) {
        if let Some(result) = &self.last_score {
            let score_line = Line::from(vec![
                Span::styled(
                    format!("  {} ", result.hand_type),
                    Style::default()
                        .fg(Theme::BRIGHT_TEXT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("\u{2502} ", Style::default().fg(Theme::CARD_BORDER)),
                Span::styled(
                    format!("{}", result.total_chips),
                    Style::default()
                        .fg(Theme::CHIPS_COLOR)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" \u{00d7} ", Style::default().fg(Theme::BRIGHT_TEXT)),
                Span::styled(
                    format!("{}", result.total_mult),
                    Style::default()
                        .fg(Theme::MULT_COLOR)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" = {}", result.final_score),
                    Style::default()
                        .fg(Theme::SCORE_COLOR)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            frame.render_widget(Paragraph::new(score_line), area);
        } else {
            // Show detected hand preview
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "  Select cards and play a hand",
                    Style::default().fg(Theme::DIM_TEXT),
                ))),
                area,
            );
        }
    }

    fn render_hand(&mut self, frame: &mut Frame, game: &RunState, area: Rect) {
        let hand_widget =
            HandWidget::new(&game.hand, &game.selected_indices).cursor(Some(self.cursor));

        // Store card rects for mouse hit-testing
        self.hand_card_rects.clear();
        for i in 0..game.hand.len() {
            if let Some(rect) = hand_widget.card_rect(area, i) {
                self.hand_card_rects.push(rect);
            } else {
                self.hand_card_rects.push(Rect::default());
            }
        }

        frame.render_widget(hand_widget, area);

        // Show hand preview above cursor position
        if !game.selected_indices.is_empty() {
            let selected_cards = game.selected_cards();
            let hand_result = detect_hand(&selected_cards);
            let preview = Line::from(vec![Span::styled(
                format!("{}", hand_result.hand_type),
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            )]);
            let preview_area = Rect::new(area.x + 2, area.y, area.width.saturating_sub(4), 1);
            frame.render_widget(
                Paragraph::new(preview).alignment(Alignment::Center),
                preview_area,
            );
        }
    }

    fn render_score_panel(&self, frame: &mut Frame, game: &RunState, area: Rect) {
        let (hand_name, hand_level, chips, mult) = if let Some(result) = &self.last_score {
            (
                format!("{}", result.hand_type),
                game.hand_levels.get_level(&result.hand_type),
                result.total_chips,
                result.total_mult,
            )
        } else if !game.selected_indices.is_empty() {
            let selected = game.selected_cards();
            let hand_result = detect_hand(&selected);
            let level = game.hand_levels.get_level(&hand_result.hand_type);
            let base_chips = game.hand_levels.chips_for(&hand_result.hand_type);
            let base_mult = game.hand_levels.mult_for(&hand_result.hand_type);
            (
                format!("{}", hand_result.hand_type),
                level,
                base_chips,
                base_mult,
            )
        } else {
            (String::new(), 1, 0, 0)
        };

        let score_widget = ScoreDisplayWidget::new(
            hand_name,
            hand_level,
            chips,
            mult,
            game.round_score,
            game.score_target,
        );
        frame.render_widget(score_widget, area);
    }

    fn render_beaten_popup(&self, frame: &mut Frame, game: &RunState, area: Rect) {
        let popup = balatrust_widgets::popup::PopupWidget::new("Blind Defeated!")
            .line(
                format!("Score: {}", game.round_score),
                Style::default()
                    .fg(Theme::SCORE_COLOR)
                    .add_modifier(Modifier::BOLD),
            )
            .line(
                format!("Target: {}", game.score_target),
                Style::default().fg(Theme::MUTED_TEXT),
            )
            .line(String::new(), Style::default())
            .line(
                format!("Reward: ${}", game.calculate_reward()),
                Style::default()
                    .fg(Theme::MONEY_COLOR)
                    .add_modifier(Modifier::BOLD),
            )
            .line(String::new(), Style::default())
            .line(
                "[Enter] Continue".to_string(),
                Style::default().fg(Theme::GOLD),
            )
            .size(50, 40);
        frame.render_widget(popup, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenAction> {
        // If blind is beaten, wait for enter
        if self.blind_just_beaten {
            if key.code == KeyCode::Enter {
                self.blind_just_beaten = false;
                self.last_score = None;
                return Some(ScreenAction::BeatBlind);
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
                self.cursor += 1;
                // Will be clamped in render
            }
            KeyCode::Char(' ') | KeyCode::Up | KeyCode::Char('k') => {
                return Some(ScreenAction::ToggleCard(self.cursor));
            }
            KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Enter => {
                return Some(ScreenAction::PlayHand);
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                return Some(ScreenAction::Discard);
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // Select all (up to 5)
                // This is handled by toggling all unselected cards
                // We'll return a series of toggles
                return None; // Handled differently
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                // Clear selection - done via multiple toggles
                return None;
            }
            _ => {}
        }
        None
    }

    pub fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
        _game: &Option<RunState>,
    ) -> Option<ScreenAction> {
        if self.blind_just_beaten {
            return None;
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let col = mouse.column;
                let row = mouse.row;

                // Check if click is on a card
                for (i, rect) in self.hand_card_rects.iter().enumerate() {
                    if col >= rect.x
                        && col < rect.x + rect.width
                        && row >= rect.y
                        && row < rect.y + rect.height
                    {
                        self.cursor = i;
                        return Some(ScreenAction::ToggleCard(i));
                    }
                }
            }
            _ => {}
        }
        None
    }

    pub fn tick(&mut self, game: &mut Option<RunState>) {
        // Clamp cursor
        if let Some(game) = game {
            if !game.hand.is_empty() && self.cursor >= game.hand.len() {
                self.cursor = game.hand.len() - 1;
            }
        }
    }
}
