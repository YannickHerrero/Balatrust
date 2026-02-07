use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use balatrust_core::hand::detect_hand;
use balatrust_core::scoring::{ScoreResult, ScoreStep};
use balatrust_core::PlayingCard;
use balatrust_core::RunState;
use balatrust_widgets::hand::HandWidget;
use balatrust_widgets::hud::HudWidget;
use balatrust_widgets::joker_bar::JokerBarWidget;
use balatrust_widgets::played_cards::PlayedCardsWidget;
use balatrust_widgets::score_display::ScoreDisplayWidget;
use balatrust_widgets::score_popup::{ScorePopup, ScorePopupKind};
use balatrust_widgets::theme::Theme;

use crate::app::ScreenAction;
use crate::effects::FxManager;

// ─── Scoring Animation State Machine ─────────────────────────────────

/// Phases of the scoring animation
#[derive(Debug, Clone)]
enum ScoringPhase {
    /// Not currently scoring - normal play mode
    NotScoring,
    /// Cards have appeared in the played zone, brief pause before scoring begins
    ShowingPlayedCards { timer: u8 },
    /// Flash the hand type label
    ShowingHandType { timer: u8 },
    /// Processing one ScoreStep at a time
    ScoringStep { step_index: usize, timer: u8 },
    /// Final chips × mult slam
    FinalScore { timer: u8 },
    /// Animation complete, waiting for app to finalize
    Done,
}

/// Ticks per animation phase
const TICKS_SHOW_PLAYED: u8 = 10;
const TICKS_SHOW_HAND_TYPE: u8 = 12;
const TICKS_PER_STEP: u8 = 8;
const TICKS_FINAL_SCORE: u8 = 12;

pub struct PlayRoundScreen {
    pub cursor: usize,
    pub last_score: Option<ScoreResult>,
    pub last_played: Vec<PlayingCard>,
    pub blind_just_beaten: bool,
    hand_card_rects: Vec<Rect>,

    // ── Scoring animation state ──
    scoring_phase: ScoringPhase,
    /// The full score result being animated
    pub scoring_result: Option<ScoreResult>,
    /// Cards currently displayed in the played zone
    played_cards: Vec<PlayingCard>,
    /// Running chips total during animation
    anim_chips: u64,
    /// Running mult total during animation
    anim_mult: f64,
    /// Which played card is currently highlighted
    active_card_index: Option<usize>,
    /// Which joker is currently activated
    active_joker_index: Option<usize>,
    /// Current popup to display: (text, kind, target_rect)
    popup: Option<(String, ScorePopupKind, Rect)>,
    /// Cached rects for played cards (computed during render)
    played_card_rects: Vec<Rect>,
    /// Cached rects for jokers (computed during render)
    joker_rects: Vec<Rect>,
    /// The hand type name for display during animation
    anim_hand_name: String,
    /// Index of joker being inspected (click-to-view detail popup)
    inspected_joker: Option<usize>,
}

impl PlayRoundScreen {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            last_score: None,
            last_played: Vec::new(),
            blind_just_beaten: false,
            hand_card_rects: Vec::new(),
            scoring_phase: ScoringPhase::NotScoring,
            scoring_result: None,
            played_cards: Vec::new(),
            anim_chips: 0,
            anim_mult: 0.0,
            active_card_index: None,
            active_joker_index: None,
            popup: None,
            played_card_rects: Vec::new(),
            joker_rects: Vec::new(),
            anim_hand_name: String::new(),
            inspected_joker: None,
        }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
        self.last_score = None;
        self.last_played.clear();
        self.blind_just_beaten = false;
        self.hand_card_rects.clear();
        self.scoring_phase = ScoringPhase::NotScoring;
        self.scoring_result = None;
        self.played_cards.clear();
        self.anim_chips = 0;
        self.anim_mult = 0.0;
        self.active_card_index = None;
        self.active_joker_index = None;
        self.popup = None;
        self.played_card_rects.clear();
        self.joker_rects.clear();
        self.anim_hand_name.clear();
        self.inspected_joker = None;
    }

    /// Returns true if we're currently in a scoring animation
    pub fn is_scoring(&self) -> bool {
        !matches!(
            self.scoring_phase,
            ScoringPhase::NotScoring | ScoringPhase::Done
        )
    }

    /// Clean up after the scoring animation finishes (called by app after FinishScoring)
    pub fn finish_scoring(&mut self) {
        self.scoring_phase = ScoringPhase::NotScoring;
        self.played_cards.clear();
        self.active_card_index = None;
        self.active_joker_index = None;
        self.popup = None;
    }

    /// Start the scoring animation with the given result and played cards
    pub fn start_scoring(&mut self, result: ScoreResult, played_cards: Vec<PlayingCard>) {
        self.anim_hand_name = format!("{}", result.hand_type);
        self.played_cards = played_cards;
        self.anim_chips = 0;
        self.anim_mult = 0.0;
        self.active_card_index = None;
        self.active_joker_index = None;
        self.popup = None;
        self.scoring_result = Some(result);
        self.scoring_phase = ScoringPhase::ShowingPlayedCards {
            timer: TICKS_SHOW_PLAYED,
        };
    }

    /// Skip the entire animation and jump to Done
    fn skip_animation(&mut self) {
        if let Some(result) = &self.scoring_result {
            self.anim_chips = result.total_chips;
            self.anim_mult = result.total_mult as f64;
        }
        self.active_card_index = None;
        self.active_joker_index = None;
        self.popup = None;
        self.scoring_phase = ScoringPhase::Done;
    }

    /// Advance the scoring state machine by one tick.
    /// Returns Some(ScreenAction::FinishScoring) when animation is complete.
    pub fn tick_scoring(&mut self, fx: &mut FxManager) -> Option<ScreenAction> {
        // Clone the phase to avoid borrow conflicts with &mut self methods
        let phase = self.scoring_phase.clone();

        match phase {
            ScoringPhase::NotScoring => None,

            ScoringPhase::ShowingPlayedCards { timer } => {
                if timer == 0 {
                    self.scoring_phase = ScoringPhase::ShowingHandType {
                        timer: TICKS_SHOW_HAND_TYPE,
                    };
                } else {
                    self.scoring_phase = ScoringPhase::ShowingPlayedCards { timer: timer - 1 };
                }
                None
            }

            ScoringPhase::ShowingHandType { timer } => {
                if timer == 0 {
                    // Apply the BaseHand step (always index 0) then advance to step 1
                    if let Some(result) = &self.scoring_result {
                        if let Some(ScoreStep::BaseHand { chips, mult, .. }) = result.steps.first()
                        {
                            self.anim_chips = *chips;
                            self.anim_mult = *mult as f64;
                        }
                    }
                    let next_index = 1; // Skip BaseHand, start from card/joker steps
                    let has_more = self
                        .scoring_result
                        .as_ref()
                        .map_or(false, |r| next_index < r.steps.len());
                    if has_more {
                        self.scoring_phase = ScoringPhase::ScoringStep {
                            step_index: next_index,
                            timer: TICKS_PER_STEP,
                        };
                    } else {
                        self.scoring_phase = ScoringPhase::FinalScore {
                            timer: TICKS_FINAL_SCORE,
                        };
                    }
                } else {
                    self.scoring_phase = ScoringPhase::ShowingHandType { timer: timer - 1 };
                }
                None
            }

            ScoringPhase::ScoringStep { step_index, timer } => {
                if timer == TICKS_PER_STEP {
                    // First tick of this step: apply the step and show popup
                    self.apply_step(step_index, fx);
                }

                if timer == 0 {
                    // Clear popup and highlights
                    self.active_card_index = None;
                    self.active_joker_index = None;
                    self.popup = None;

                    let next = step_index + 1;
                    let has_more = self
                        .scoring_result
                        .as_ref()
                        .map_or(false, |r| next < r.steps.len());
                    if has_more {
                        self.scoring_phase = ScoringPhase::ScoringStep {
                            step_index: next,
                            timer: TICKS_PER_STEP,
                        };
                    } else {
                        self.scoring_phase = ScoringPhase::FinalScore {
                            timer: TICKS_FINAL_SCORE,
                        };
                    }
                } else {
                    self.scoring_phase = ScoringPhase::ScoringStep {
                        step_index,
                        timer: timer - 1,
                    };
                }
                None
            }

            ScoringPhase::FinalScore { timer } => {
                if timer == 0 {
                    self.scoring_phase = ScoringPhase::Done;
                    // Signal to app that scoring animation is complete
                    return Some(ScreenAction::FinishScoring);
                } else {
                    self.scoring_phase = ScoringPhase::FinalScore { timer: timer - 1 };
                }
                None
            }

            ScoringPhase::Done => None,
        }
    }

    /// Apply a single ScoreStep: update running totals, set highlights, create popup
    fn apply_step(&mut self, step_index: usize, fx: &mut FxManager) {
        let step = match &self.scoring_result {
            Some(r) => r.steps.get(step_index).cloned(),
            None => return,
        };

        let step = match step {
            Some(s) => s,
            None => return,
        };

        let popup_text = step.popup_text();
        let popup_kind = match step.popup_kind() {
            "chips" => ScorePopupKind::Chips,
            "mult" => ScorePopupKind::Mult,
            "xmult" => ScorePopupKind::XMult,
            _ => ScorePopupKind::Chips,
        };

        match &step {
            ScoreStep::CardChips { card_index, chips } => {
                self.active_card_index = Some(*card_index);
                self.active_joker_index = None;
                self.anim_chips += chips;
                self.set_popup_at_card(*card_index, popup_text, popup_kind);
                // Fire tachyonfx glow on the card rect
                if let Some(rect) = self.played_card_rects.get(*card_index).copied() {
                    fx.add_unique_effect(
                        format!("card_score_{}", card_index),
                        crate::effects::card_score_glow().with_area(rect),
                    );
                }
            }
            ScoreStep::CardMult { card_index, .. } | ScoreStep::CardXMult { card_index, .. } => {
                self.active_card_index = Some(*card_index);
                self.active_joker_index = None;
                match &step {
                    ScoreStep::CardMult { mult, .. } => self.anim_mult += *mult as f64,
                    ScoreStep::CardXMult { x_mult, .. } => self.anim_mult *= x_mult,
                    _ => {}
                }
                self.set_popup_at_card(*card_index, popup_text, popup_kind);
                if let Some(rect) = self.played_card_rects.get(*card_index).copied() {
                    fx.add_unique_effect(
                        format!("card_score_{}", card_index),
                        crate::effects::card_score_glow().with_area(rect),
                    );
                }
            }
            ScoreStep::JokerChips {
                joker_index, chips, ..
            } => {
                self.active_card_index = None;
                self.active_joker_index = Some(*joker_index);
                self.anim_chips += chips;
                self.set_popup_at_joker(*joker_index, popup_text, popup_kind);
                if let Some(rect) = self.joker_rects.get(*joker_index).copied() {
                    fx.add_unique_effect(
                        format!("joker_activate_{}", joker_index),
                        crate::effects::joker_activate_pulse().with_area(rect),
                    );
                }
            }
            ScoreStep::JokerMult { joker_index, mult } => {
                self.active_card_index = None;
                self.active_joker_index = Some(*joker_index);
                self.anim_mult += *mult as f64;
                self.set_popup_at_joker(*joker_index, popup_text, popup_kind);
                if let Some(rect) = self.joker_rects.get(*joker_index).copied() {
                    fx.add_unique_effect(
                        format!("joker_activate_{}", joker_index),
                        crate::effects::joker_activate_pulse().with_area(rect),
                    );
                }
            }
            ScoreStep::JokerXMult {
                joker_index,
                x_mult,
            } => {
                self.active_card_index = None;
                self.active_joker_index = Some(*joker_index);
                self.anim_mult *= x_mult;
                self.set_popup_at_joker(*joker_index, popup_text, popup_kind);
                if let Some(rect) = self.joker_rects.get(*joker_index).copied() {
                    fx.add_unique_effect(
                        format!("joker_activate_{}", joker_index),
                        crate::effects::joker_activate_pulse().with_area(rect),
                    );
                }
            }
            ScoreStep::JokerCardChips {
                joker_index,
                card_index,
                chips,
            } => {
                self.active_card_index = Some(*card_index);
                self.active_joker_index = Some(*joker_index);
                self.anim_chips += chips;
                // Popup appears at the card since it was triggered per-card
                self.set_popup_at_card(*card_index, popup_text, popup_kind);
                if let Some(rect) = self.joker_rects.get(*joker_index).copied() {
                    fx.add_unique_effect(
                        format!("joker_activate_{}", joker_index),
                        crate::effects::joker_activate_pulse().with_area(rect),
                    );
                }
            }
            ScoreStep::JokerCardMult {
                joker_index,
                card_index,
                mult,
            } => {
                self.active_card_index = Some(*card_index);
                self.active_joker_index = Some(*joker_index);
                self.anim_mult += *mult as f64;
                self.set_popup_at_card(*card_index, popup_text, popup_kind);
                if let Some(rect) = self.joker_rects.get(*joker_index).copied() {
                    fx.add_unique_effect(
                        format!("joker_activate_{}", joker_index),
                        crate::effects::joker_activate_pulse().with_area(rect),
                    );
                }
            }
            ScoreStep::BaseHand { .. } => {
                // Already handled in ShowingHandType phase
            }
        }
    }

    fn set_popup_at_card(&mut self, card_index: usize, text: String, kind: ScorePopupKind) {
        if let Some(rect) = self.played_card_rects.get(card_index).copied() {
            self.popup = Some((text, kind, rect));
        }
    }

    fn set_popup_at_joker(&mut self, joker_index: usize, text: String, kind: ScorePopupKind) {
        if let Some(rect) = self.joker_rects.get(joker_index).copied() {
            self.popup = Some((text, kind, rect));
        }
    }

    // ─── Rendering ───────────────────────────────────────────────────

    pub fn render(&mut self, frame: &mut Frame, game: &Option<RunState>) {
        let area = frame.area();

        let game = match game {
            Some(g) => g,
            None => return,
        };

        let is_scoring = self.is_scoring();

        // Main layout changes based on whether we're scoring
        let main_chunks = if is_scoring {
            Layout::vertical([
                Constraint::Length(3), // Header (blind info)
                Constraint::Length(5), // Joker bar
                Constraint::Length(8), // Played cards zone
                Constraint::Length(2), // Score info (animated chips × mult)
                Constraint::Min(0),    // Hand area (smaller during scoring)
                Constraint::Length(1), // HUD
                Constraint::Length(2), // Help
            ])
            .split(area)
        } else {
            Layout::vertical([
                Constraint::Length(3), // Header (blind info)
                Constraint::Length(5), // Joker bar
                Constraint::Length(2), // Last played cards info
                Constraint::Min(0),    // Hand area
                Constraint::Length(1), // HUD
                Constraint::Length(2), // Help
            ])
            .split(area)
        };

        // === Header: Blind info ===
        self.render_header(frame, game, main_chunks[0]);

        // === Joker bar ===
        let joker_bar =
            JokerBarWidget::new(&game.jokers, game.max_jokers).activated(self.active_joker_index);

        // Cache joker rects before rendering
        self.joker_rects.clear();
        for i in 0..game.jokers.len() {
            if let Some(rect) = joker_bar.joker_rect(main_chunks[1], i) {
                self.joker_rects.push(rect);
            } else {
                self.joker_rects.push(Rect::default());
            }
        }
        frame.render_widget(joker_bar, main_chunks[1]);

        if is_scoring {
            // === Played cards zone ===
            self.render_played_cards(frame, main_chunks[2]);

            // === Animated score info ===
            self.render_animated_score(frame, main_chunks[3]);

            // === Hand area (dimmed during scoring) ===
            let game_area = Layout::horizontal([
                Constraint::Min(0),     // Hand cards
                Constraint::Length(22), // Score panel
            ])
            .split(main_chunks[4]);

            self.render_hand(frame, game, game_area[0]);
            self.render_score_panel(frame, game, game_area[1]);

            // === Popup overlay (rendered last so it appears on top) ===
            self.render_popup(frame);

            // === HUD ===
            let hud = HudWidget::new(
                game.hands_remaining,
                game.discards_remaining,
                game.money,
                game.deck.remaining(),
            )
            .can_play(false)
            .can_discard(false);
            frame.render_widget(hud, main_chunks[5]);

            // === Help line (scoring) ===
            let help = Paragraph::new(Line::from(vec![
                Span::styled("[", Style::default().fg(Theme::DIM_TEXT)),
                Span::styled("Space/Enter", Style::default().fg(Theme::GOLD)),
                Span::styled("] Skip Animation", Style::default().fg(Theme::DIM_TEXT)),
            ]))
            .alignment(Alignment::Center);
            frame.render_widget(help, main_chunks[6]);
        } else {
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
        }

        // Joker inspect popup (rendered on top of everything except blind beaten)
        if let Some(ji) = self.inspected_joker {
            self.render_joker_inspect(frame, game, ji, area);
        }

        // Blind beaten popup (always on top)
        if self.blind_just_beaten {
            self.render_beaten_popup(frame, game, area);
        }
    }

    fn render_played_cards(&mut self, frame: &mut Frame, area: Rect) {
        let scoring_indices = self
            .scoring_result
            .as_ref()
            .map(|r| r.scoring_indices.as_slice())
            .unwrap_or(&[]);

        let widget = PlayedCardsWidget::new(&self.played_cards, scoring_indices)
            .active_card(self.active_card_index);

        // Cache card rects for popup placement
        self.played_card_rects.clear();
        for i in 0..self.played_cards.len() {
            if let Some(rect) = widget.card_rect(area, i) {
                self.played_card_rects.push(rect);
            } else {
                self.played_card_rects.push(Rect::default());
            }
        }

        frame.render_widget(widget, area);
    }

    fn render_animated_score(&self, frame: &mut Frame, area: Rect) {
        let chips_display = self.anim_chips;
        let mult_display = self.anim_mult.max(0.0).ceil() as u64;

        let is_final = matches!(
            self.scoring_phase,
            ScoringPhase::FinalScore { .. } | ScoringPhase::Done
        );

        let line = Line::from(vec![
            Span::styled(
                format!("  {} ", self.anim_hand_name),
                Style::default()
                    .fg(Theme::BRIGHT_TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("\u{2502} ", Style::default().fg(Theme::CARD_BORDER)),
            Span::styled(
                format!("{}", chips_display),
                Style::default()
                    .fg(Theme::CHIPS_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" \u{00d7} ", Style::default().fg(Theme::BRIGHT_TEXT)),
            Span::styled(
                format!("{}", mult_display),
                Style::default()
                    .fg(Theme::MULT_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            if is_final {
                Span::styled(
                    format!(" = {}", chips_display * mult_display),
                    Style::default()
                        .fg(Theme::SCORE_COLOR)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("")
            },
        ]);
        frame.render_widget(Paragraph::new(line), area);
    }

    fn render_popup(&self, frame: &mut Frame) {
        if let Some((ref text, kind, target_rect)) = self.popup {
            let popup = ScorePopup::new(text.clone(), kind);
            // Render above the target rect
            frame.render_widget(popup, target_rect);
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

        // Show hand preview above cursor position (only when not scoring)
        if !self.is_scoring() && !game.selected_indices.is_empty() {
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
        let (hand_name, hand_level, chips, mult) = if let Some(result) = &self.scoring_result {
            // During animation, show the animated running totals
            (
                self.anim_hand_name.clone(),
                game.hand_levels.get_level(&result.hand_type),
                self.anim_chips,
                self.anim_mult.max(0.0).ceil() as u64,
            )
        } else if let Some(result) = &self.last_score {
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

    fn render_joker_inspect(
        &self,
        frame: &mut Frame,
        game: &RunState,
        joker_index: usize,
        _screen_area: Rect,
    ) {
        let joker = match game.jokers.get(joker_index) {
            Some(j) => j,
            None => {
                return;
            }
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

        // Build lines for the popup
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

        // Size the popup to fit content
        let content_width = lines.iter().map(|l| l.width() as u16).max().unwrap_or(10) + 4; // +4 for border + padding
        let popup_width = content_width.max(20).min(40);
        let popup_height = (lines.len() as u16) + 3; // +3 for border top/bottom + padding

        // Position below the joker card, centered on it
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

        // Clear background and draw border
        frame.render_widget(ratatui::widgets::Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(rarity_color))
            .padding(ratatui::widgets::Padding::horizontal(1));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Render lines
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

    // ─── Input Handling ──────────────────────────────────────────────

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

        // Dismiss joker inspect popup
        if self.inspected_joker.is_some() {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ')) {
                self.inspected_joker = None;
            }
            return None;
        }

        // During scoring animation, only Space/Enter to skip
        if self.is_scoring() {
            if matches!(
                key.code,
                KeyCode::Char(' ') | KeyCode::Enter | KeyCode::Char('p') | KeyCode::Char('P')
            ) {
                self.skip_animation();
                return Some(ScreenAction::FinishScoring);
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
                return None; // Handled differently
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                // Clear selection
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

                // Check if click is on a joker (works during scoring too)
                for (i, rect) in self.joker_rects.iter().enumerate() {
                    if rect.width > 0
                        && col >= rect.x
                        && col < rect.x + rect.width
                        && row >= rect.y
                        && row < rect.y + rect.height
                    {
                        // Toggle: click same joker again to dismiss
                        if self.inspected_joker == Some(i) {
                            self.inspected_joker = None;
                        } else {
                            self.inspected_joker = Some(i);
                        }
                        return None;
                    }
                }

                // Click elsewhere dismisses the joker popup
                if self.inspected_joker.is_some() {
                    self.inspected_joker = None;
                    return None;
                }

                // Don't process other clicks during scoring
                if self.is_scoring() {
                    return None;
                }

                // Check if click is on a hand card
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
