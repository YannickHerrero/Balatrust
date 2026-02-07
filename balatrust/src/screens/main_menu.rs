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

pub struct MainMenuScreen {
    pub selected: usize,
}

impl MainMenuScreen {
    pub fn new() -> Self {
        Self { selected: 0 }
    }
}

impl Screen for MainMenuScreen {
    fn render(&mut self, frame: &mut Frame, _game: &Option<RunState>) {
        let area = frame.area();

        let chunks = Layout::vertical([
            Constraint::Percentage(30),
            Constraint::Length(12),
            Constraint::Percentage(30),
            Constraint::Min(3),
        ])
        .split(area);

        // Title - ASCII art
        let title_lines = vec![
            Line::from(Span::styled(
                " ____        _       _                   _   ",
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "| __ )  __ _| | __ _| |_ _ __ _   _ ___| |_ ",
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "|  _ \\ / _` | |/ _` | __| '__| | | / __| __|",
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "| |_) | (_| | | (_| | |_| |  | |_| \\__ \\ |_ ",
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "|____/ \\__,_|_|\\__,_|\\__|_|   \\__,_|___/\\__|",
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "A terminal Balatro experience",
                Style::default().fg(Theme::MUTED_TEXT),
            )),
        ];

        let title = Paragraph::new(title_lines).alignment(Alignment::Center);
        frame.render_widget(title, chunks[1]);

        // Menu options
        let menu_items = vec!["New Game", "Quit"];
        let mut menu_lines = Vec::new();
        for (i, item) in menu_items.iter().enumerate() {
            let style = if i == self.selected {
                Style::default()
                    .fg(Theme::CARD_SELECTED)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::MUTED_TEXT)
            };
            let prefix = if i == self.selected { "> " } else { "  " };
            menu_lines.push(Line::from(Span::styled(
                format!("{}{}", prefix, item),
                style,
            )));
        }

        let menu = Paragraph::new(menu_lines).alignment(Alignment::Center);
        frame.render_widget(menu, chunks[2]);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("[", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("\u{2191}\u{2193}", Style::default().fg(Theme::GOLD)),
            Span::styled("] Navigate  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("Enter", Style::default().fg(Theme::GOLD)),
            Span::styled("] Select  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("q", Style::default().fg(Theme::GOLD)),
            Span::styled("] Quit", Style::default().fg(Theme::DIM_TEXT)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[3]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenAction> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected < 1 {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => {
                return match self.selected {
                    0 => Some(ScreenAction::NewGame),
                    1 => Some(ScreenAction::Quit),
                    _ => None,
                };
            }
            KeyCode::Char('q') => return Some(ScreenAction::Quit),
            _ => {}
        }
        None
    }
}
