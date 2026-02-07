use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use balatrust_core::RunState;
use balatrust_widgets::theme::Theme;

use crate::app::ScreenAction;
use crate::screens::Screen;

pub struct GameOverScreen {
    pub won: bool,
}

impl GameOverScreen {
    pub fn new() -> Self {
        Self { won: false }
    }
}

impl Screen for GameOverScreen {
    fn render(&mut self, frame: &mut Frame, game: &Option<RunState>) {
        let area = frame.area();

        let chunks = Layout::vertical([
            Constraint::Percentage(30),
            Constraint::Length(10),
            Constraint::Percentage(30),
            Constraint::Min(3),
        ])
        .split(area);

        let (title, title_color) = if self.won {
            ("YOU WIN!", Theme::GOLD)
        } else {
            ("GAME OVER", Theme::MULT_COLOR)
        };

        let mut lines = vec![
            Line::from(Span::styled(
                title,
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        if let Some(game) = game {
            lines.push(Line::from(Span::styled(
                format!("Ante: {}", game.ante),
                Style::default().fg(Theme::BRIGHT_TEXT),
            )));
            lines.push(Line::from(Span::styled(
                format!("Money: ${}", game.money),
                Style::default().fg(Theme::MONEY_COLOR),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[Enter] New Game  [Q] Quit",
            Style::default().fg(Theme::GOLD),
        )));

        let content = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(content, chunks[1]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenAction> {
        match key.code {
            KeyCode::Enter => Some(ScreenAction::NewGame),
            KeyCode::Char('q') | KeyCode::Char('Q') => Some(ScreenAction::Quit),
            KeyCode::Esc => Some(ScreenAction::BackToMenu),
            _ => None,
        }
    }
}
