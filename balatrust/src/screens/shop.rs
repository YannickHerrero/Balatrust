use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use balatrust_core::RunState;
use balatrust_widgets::theme::Theme;

use crate::app::ScreenAction;
use crate::screens::Screen;

pub struct ShopScreen {
    pub cursor: usize,
}

impl ShopScreen {
    pub fn new() -> Self {
        Self { cursor: 0 }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }
}

impl Screen for ShopScreen {
    fn render(&mut self, frame: &mut Frame, game: &Option<RunState>) {
        let area = frame.area();
        let bg = Block::default().style(Style::default().bg(Theme::BG));
        frame.render_widget(bg, area);

        let game = match game {
            Some(g) => g,
            None => return,
        };

        let chunks = Layout::vertical([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(area);

        // Header
        let header = Line::from(vec![
            Span::styled(
                format!("  ${}  ", game.money),
                Style::default()
                    .fg(Theme::MONEY_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "SHOP",
                Style::default()
                    .fg(Theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  Ante {} / {}  ", game.ante, game.blind_type),
                Style::default().fg(Theme::MUTED_TEXT),
            ),
        ]);
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Theme::CARD_BORDER));
        let header_inner = header_block.inner(chunks[0]);
        frame.render_widget(header_block, chunks[0]);
        frame.render_widget(
            Paragraph::new(header).alignment(Alignment::Center),
            header_inner,
        );

        // Shop content - placeholder for now
        let content = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "Shop coming soon!",
                Style::default().fg(Theme::MUTED_TEXT),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Jokers, Planet cards, and Tarot cards",
                Style::default().fg(Theme::DIM_TEXT),
            )),
            Line::from(Span::styled(
                "will appear here for purchase.",
                Style::default().fg(Theme::DIM_TEXT),
            )),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                format!("Interest: +${}", (game.money / 5).min(5)),
                Style::default().fg(Theme::MONEY_COLOR),
            )),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(content, chunks[1]);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("[", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("N", Style::default().fg(Theme::GOLD)),
            Span::styled("] Next Round  [", Style::default().fg(Theme::DIM_TEXT)),
            Span::styled("Enter", Style::default().fg(Theme::GOLD)),
            Span::styled("] Next Round", Style::default().fg(Theme::DIM_TEXT)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenAction> {
        match key.code {
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Enter => {
                return Some(ScreenAction::LeaveShop);
            }
            _ => {}
        }
        None
    }
}
