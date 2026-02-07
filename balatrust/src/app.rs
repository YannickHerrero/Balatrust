use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::Frame;

use balatrust_core::RunState;

use crate::screens::blind_select::BlindSelectScreen;
use crate::screens::game_over::GameOverScreen;
use crate::screens::main_menu::MainMenuScreen;
use crate::screens::play_round::PlayRoundScreen;
use crate::screens::shop::ShopScreen;
use crate::screens::Screen;

/// Top-level game phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    MainMenu,
    BlindSelect,
    Playing,
    Shop,
    GameOver { won: bool },
}

/// Main application state
pub struct App {
    pub phase: GamePhase,
    pub game: Option<RunState>,

    // Screens
    pub main_menu: MainMenuScreen,
    pub blind_select: BlindSelectScreen,
    pub play_round: PlayRoundScreen,
    pub shop: ShopScreen,
    pub game_over: GameOverScreen,
}

impl App {
    pub fn new() -> Self {
        Self {
            phase: GamePhase::MainMenu,
            game: None,
            main_menu: MainMenuScreen::new(),
            blind_select: BlindSelectScreen::new(),
            play_round: PlayRoundScreen::new(),
            shop: ShopScreen::new(),
            game_over: GameOverScreen::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        match self.phase {
            GamePhase::MainMenu => self.main_menu.render(frame, &self.game),
            GamePhase::BlindSelect => self.blind_select.render(frame, &self.game),
            GamePhase::Playing => self.play_round.render(frame, &self.game),
            GamePhase::Shop => self.shop.render(frame, &self.game),
            GamePhase::GameOver { won } => {
                self.game_over.won = won;
                self.game_over.render(frame, &self.game);
            }
        }
    }

    /// Handle key event. Returns true if should quit.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Global quit
        if key.code == KeyCode::Char('q') && self.phase == GamePhase::MainMenu {
            return true;
        }

        let action = match self.phase {
            GamePhase::MainMenu => self.main_menu.handle_key(key),
            GamePhase::BlindSelect => self.blind_select.handle_key(key),
            GamePhase::Playing => self.play_round.handle_key(key),
            GamePhase::Shop => self.shop.handle_key(key),
            GamePhase::GameOver { .. } => self.game_over.handle_key(key),
        };

        self.process_action(action)
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        match self.phase {
            GamePhase::Playing => {
                let action = self.play_round.handle_mouse(mouse, &self.game);
                self.process_action(action);
            }
            _ => {}
        }
    }

    pub fn handle_resize(&mut self, _w: u16, _h: u16) {
        // Ratatui handles resize automatically
    }

    pub fn tick(&mut self) {
        match self.phase {
            GamePhase::Playing => self.play_round.tick(&mut self.game),
            _ => {}
        }
    }

    /// Process a screen action. Returns true if should quit.
    fn process_action(&mut self, action: Option<ScreenAction>) -> bool {
        match action {
            Some(ScreenAction::Quit) => return true,
            Some(ScreenAction::NewGame) => {
                self.game = Some(RunState::new());
                self.phase = GamePhase::BlindSelect;
            }
            Some(ScreenAction::StartBlind) => {
                if let Some(game) = &mut self.game {
                    game.start_blind();
                    self.play_round.reset();
                    self.phase = GamePhase::Playing;
                }
            }
            Some(ScreenAction::SkipBlind) => {
                if let Some(game) = &mut self.game {
                    game.skip_blind();
                    self.blind_select.cursor = 0;
                    // Stay on blind select
                }
            }
            Some(ScreenAction::PlayHand) => {
                if let Some(game) = &mut self.game {
                    if game.can_play() {
                        let played = game.play_selected();
                        let score_result = balatrust_core::scoring::calculate_score_with_jokers(
                            &played,
                            &game.hand_levels,
                            &game.jokers,
                            &game.hand, // remaining hand = held cards
                            game.discards_remaining,
                        );
                        game.add_score(score_result.final_score);
                        game.use_hand();

                        // Store the last result for display
                        self.play_round.last_score = Some(score_result);
                        self.play_round.last_played = played;

                        // Draw replacement cards
                        game.draw_to_hand_size();

                        // Check win/lose
                        if game.blind_beaten() {
                            self.play_round.blind_just_beaten = true;
                        } else if game.round_lost() {
                            self.phase = GamePhase::GameOver { won: false };
                        }
                    }
                }
            }
            Some(ScreenAction::Discard) => {
                if let Some(game) = &mut self.game {
                    if game.can_discard() {
                        game.use_discard();
                        game.discard_selected();
                    }
                }
            }
            Some(ScreenAction::BeatBlind) => {
                if let Some(game) = &mut self.game {
                    game.beat_blind();
                    if game.run_won() {
                        self.phase = GamePhase::GameOver { won: true };
                    } else {
                        self.phase = GamePhase::Shop;
                        self.shop.reset();
                    }
                }
            }
            Some(ScreenAction::LeaveShop) => {
                if let Some(game) = &mut self.game {
                    game.leave_shop();
                    self.blind_select.cursor = 0;
                    self.phase = GamePhase::BlindSelect;
                }
            }
            Some(ScreenAction::BackToMenu) => {
                self.game = None;
                self.phase = GamePhase::MainMenu;
            }
            Some(ScreenAction::ToggleCard(idx)) => {
                if let Some(game) = &mut self.game {
                    game.toggle_select(idx);
                }
            }
            Some(ScreenAction::BuyShopItem(idx)) => {
                if let Some(game) = &mut self.game {
                    game.buy_shop_item(idx);
                }
            }
            Some(ScreenAction::SellJoker(idx)) => {
                if let Some(game) = &mut self.game {
                    game.sell_joker(idx);
                }
            }
            Some(ScreenAction::RerollShop) => {
                if let Some(game) = &mut self.game {
                    game.reroll_shop();
                }
            }
            Some(ScreenAction::UseConsumable(idx)) => {
                if let Some(game) = &mut self.game {
                    // Try planet first, then tarot
                    if !game.use_planet(idx) {
                        game.use_tarot(idx);
                    }
                }
            }
            None => {}
        }

        false
    }
}

/// Actions that screens can return
#[derive(Debug, Clone)]
pub enum ScreenAction {
    Quit,
    NewGame,
    StartBlind,
    SkipBlind,
    PlayHand,
    Discard,
    BeatBlind,
    LeaveShop,
    BackToMenu,
    ToggleCard(usize),
    BuyShopItem(usize),
    SellJoker(usize),
    RerollShop,
    UseConsumable(usize),
}
