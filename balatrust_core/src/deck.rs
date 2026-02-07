use rand::seq::SliceRandom;
use rand::Rng;

use crate::card::{PlayingCard, Rank, Suit};

#[derive(Debug, Clone)]
pub struct Deck {
    cards: Vec<PlayingCard>,
    discard: Vec<PlayingCard>,
}

impl Deck {
    /// Create a standard 52-card deck
    pub fn standard() -> Self {
        let mut cards = Vec::with_capacity(52);
        for &suit in &Suit::ALL {
            for &rank in &Rank::ALL {
                cards.push(PlayingCard::new(rank, suit));
            }
        }
        Self {
            cards,
            discard: Vec::new(),
        }
    }

    /// Shuffle the draw pile
    pub fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        self.cards.shuffle(rng);
    }

    /// Draw n cards from the top. Returns fewer if deck runs low.
    pub fn draw(&mut self, n: usize) -> Vec<PlayingCard> {
        let mut drawn = Vec::with_capacity(n);
        for _ in 0..n {
            if let Some(card) = self.cards.pop() {
                drawn.push(card);
            } else {
                break;
            }
        }
        drawn
    }

    /// Put cards into the discard pile
    pub fn discard_cards(&mut self, cards: &[PlayingCard]) {
        self.discard.extend_from_slice(cards);
    }

    /// Shuffle the discard pile back into the draw pile
    pub fn reshuffle_discard<R: Rng>(&mut self, rng: &mut R) {
        self.cards.append(&mut self.discard);
        self.shuffle(rng);
    }

    /// Cards remaining in draw pile
    pub fn remaining(&self) -> usize {
        self.cards.len()
    }

    /// Cards in discard pile
    pub fn discard_count(&self) -> usize {
        self.discard.len()
    }

    /// Total cards in the deck (draw + discard)
    pub fn total(&self) -> usize {
        self.cards.len() + self.discard.len()
    }

    /// Reset: gather all cards, re-create a full deck with any modifications preserved
    pub fn reset_and_shuffle<R: Rng>(&mut self, rng: &mut R) {
        self.cards.append(&mut self.discard);
        self.shuffle(rng);
    }

    /// Add a card to the deck
    pub fn add_card(&mut self, card: PlayingCard) {
        self.cards.push(card);
    }

    /// Remove a card from the deck (first match)
    pub fn remove_card(&mut self, card: &PlayingCard) -> bool {
        if let Some(pos) = self.cards.iter().position(|c| c == card) {
            self.cards.remove(pos);
            true
        } else if let Some(pos) = self.discard.iter().position(|c| c == card) {
            self.discard.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_deck_has_52_cards() {
        let deck = Deck::standard();
        assert_eq!(deck.total(), 52);
    }

    #[test]
    fn test_draw_reduces_remaining() {
        let mut deck = Deck::standard();
        let drawn = deck.draw(5);
        assert_eq!(drawn.len(), 5);
        assert_eq!(deck.remaining(), 47);
    }

    #[test]
    fn test_discard_and_reshuffle() {
        let mut rng = rand::thread_rng();
        let mut deck = Deck::standard();
        let drawn = deck.draw(5);
        deck.discard_cards(&drawn);
        assert_eq!(deck.discard_count(), 5);
        assert_eq!(deck.remaining(), 47);
        deck.reshuffle_discard(&mut rng);
        assert_eq!(deck.remaining(), 52);
        assert_eq!(deck.discard_count(), 0);
    }
}
