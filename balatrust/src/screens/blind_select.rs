use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::Frame;

use balatrust_core::RunState;
use balatrust_widgets::blind_select::BlindSelectWidget;
use balatrust_widgets::theme::Theme;

use crate::app::ScreenAction;
use crate::screens::Screen;

pub struct BlindSelectScreen {
    pub cursor: usize, // 0=small, 1=big, 2=boss
}

impl BlindSelectScreen {
    pub fn new() -> Self {
        Self { cursor: 0 }
    }
}

impl Screen for BlindSelectScreen {
    fn render(&mut self, frame: &mut Frame, game: &Option<RunState>) {
        let area = frame.area();
        let bg = Block::default().style(Style::default().bg(Theme::BG));
        frame.render_widget(bg, area);

        if let Some(game) = game {
            let chunks = Layout::vertical([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

            let widget =
                BlindSelectWidget::new(game.ante, game.boss_blind, self.cursor, game.money);
            frame.render_widget(widget, chunks[1]);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenAction> {
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
            KeyCode::Enter => {
                return Some(ScreenAction::StartBlind);
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Skip (only for small and big)
                if self.cursor < 2 {
                    return Some(ScreenAction::SkipBlind);
                }
            }
            _ => {}
        }
        None
    }
}
