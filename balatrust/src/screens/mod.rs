pub mod blind_select;
pub mod game_over;
pub mod main_menu;
pub mod play_round;
pub mod shop;

use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::app::ScreenAction;
use balatrust_core::RunState;

/// Trait for game screens
pub trait Screen {
    fn render(&mut self, frame: &mut Frame, game: &Option<RunState>);
    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenAction>;
}
