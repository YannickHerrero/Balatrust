use std::collections::HashMap;

use crate::card::PlayingCard;
use crate::hand::{detect_hand, PokerHand};

/// A single step in the scoring process, used for animation
#[derive(Debug, Clone)]
pub enum ScoreStep {
    /// The base hand type contributes chips and mult
    BaseHand {
        hand_type: PokerHand,
        chips: u64,
        mult: u64,
    },
    /// A played card adds its chip value
    CardChips { card_index: usize, chips: u64 },
    /// A played card adds flat mult (enhancement/edition)
    CardMult { card_index: usize, mult: u64 },
    /// A played card applies multiplicative mult (glass/polychrome)
    CardXMult { card_index: usize, x_mult: f64 },
    /// A joker adds flat chips
    JokerChips { joker_index: usize, chips: u64 },
    /// A joker adds flat mult
    JokerMult { joker_index: usize, mult: u64 },
    /// A joker applies multiplicative mult
    JokerXMult { joker_index: usize, x_mult: f64 },
}

/// Result of scoring a hand
#[derive(Debug, Clone)]
pub struct ScoreResult {
    pub hand_type: PokerHand,
    /// Indices of cards that are part of the poker hand (scoring cards)
    pub scoring_indices: Vec<usize>,
    pub steps: Vec<ScoreStep>,
    pub total_chips: u64,
    pub total_mult: u64,
    pub final_score: u64,
}

/// Hand level state: tracks the level of each poker hand
#[derive(Debug, Clone)]
pub struct HandLevels {
    levels: HashMap<PokerHand, u8>,
}

impl Default for HandLevels {
    fn default() -> Self {
        Self::new()
    }
}

impl HandLevels {
    pub fn new() -> Self {
        let mut levels = HashMap::new();
        for hand in PokerHand::ALL {
            levels.insert(hand, 1);
        }
        Self { levels }
    }

    pub fn get_level(&self, hand: &PokerHand) -> u8 {
        *self.levels.get(hand).unwrap_or(&1)
    }

    pub fn level_up(&mut self, hand: PokerHand) {
        let entry = self.levels.entry(hand).or_insert(1);
        *entry += 1;
    }

    /// Get chips for a hand at its current level
    pub fn chips_for(&self, hand: &PokerHand) -> u64 {
        let level = self.get_level(hand);
        hand.base_chips() + (level as u64 - 1) * hand.level_up_chips()
    }

    /// Get mult for a hand at its current level
    pub fn mult_for(&self, hand: &PokerHand) -> u64 {
        let level = self.get_level(hand);
        hand.base_mult() + (level as u64 - 1) * hand.level_up_mult()
    }
}

/// Calculate the score for a set of played cards.
/// This is the core scoring function without joker effects.
pub fn calculate_score(played_cards: &[PlayingCard], hand_levels: &HandLevels) -> ScoreResult {
    let hand_result = detect_hand(played_cards);
    let hand_type = hand_result.hand_type;
    let scoring_indices = hand_result.scoring_indices;

    let mut steps = Vec::new();

    // Step 1: Base hand chips and mult
    let base_chips = hand_levels.chips_for(&hand_type);
    let base_mult = hand_levels.mult_for(&hand_type);

    steps.push(ScoreStep::BaseHand {
        hand_type,
        chips: base_chips,
        mult: base_mult,
    });

    let mut total_chips = base_chips;
    let mut total_mult_f: f64 = base_mult as f64;

    // Step 2: Process each scoring card
    for &idx in &scoring_indices {
        let card = &played_cards[idx];

        // Card chip value
        let card_chips = card.chip_value();
        if card_chips > 0 {
            steps.push(ScoreStep::CardChips {
                card_index: idx,
                chips: card_chips,
            });
            total_chips += card_chips;
        }

        // Card flat mult bonus
        let card_mult = card.mult_bonus();
        if card_mult > 0 {
            steps.push(ScoreStep::CardMult {
                card_index: idx,
                mult: card_mult,
            });
            total_mult_f += card_mult as f64;
        }

        // Card multiplicative mult
        let card_x_mult = card.x_mult();
        if (card_x_mult - 1.0).abs() > f64::EPSILON {
            steps.push(ScoreStep::CardXMult {
                card_index: idx,
                x_mult: card_x_mult,
            });
            total_mult_f *= card_x_mult;
        }
    }

    // Also process cards that always score (Stone cards) but aren't in the hand
    for (idx, card) in played_cards.iter().enumerate() {
        if card.always_scores() && !scoring_indices.contains(&idx) {
            let card_chips = card.chip_value();
            if card_chips > 0 {
                steps.push(ScoreStep::CardChips {
                    card_index: idx,
                    chips: card_chips,
                });
                total_chips += card_chips;
            }
        }
    }

    let total_mult = total_mult_f.ceil() as u64;
    let final_score = total_chips * total_mult;

    ScoreResult {
        hand_type,
        scoring_indices,
        steps,
        total_chips,
        total_mult,
        final_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Rank::*, Suit::*};
    use crate::PlayingCard;

    fn c(rank: crate::Rank, suit: crate::Suit) -> PlayingCard {
        PlayingCard::new(rank, suit)
    }

    #[test]
    fn test_pair_scoring() {
        let cards = vec![c(King, Spades), c(King, Hearts), c(Five, Clubs)];
        let levels = HandLevels::new();
        let result = calculate_score(&cards, &levels);

        assert_eq!(result.hand_type, PokerHand::Pair);
        // Base: 10 chips, 2 mult
        // Scoring cards: two Kings = +10 + 10 = 20 chips
        // Total: (10 + 20) chips * 2 mult = 60
        assert_eq!(result.total_chips, 30);
        assert_eq!(result.total_mult, 2);
        assert_eq!(result.final_score, 60);
    }

    #[test]
    fn test_flush_scoring() {
        let cards = vec![
            c(Two, Hearts),
            c(Five, Hearts),
            c(Eight, Hearts),
            c(Jack, Hearts),
            c(Ace, Hearts),
        ];
        let levels = HandLevels::new();
        let result = calculate_score(&cards, &levels);

        assert_eq!(result.hand_type, PokerHand::Flush);
        // Base: 35 chips, 4 mult
        // All 5 cards score: 2+5+8+10+11 = 36
        // Total: (35 + 36) * 4 = 284
        assert_eq!(result.total_chips, 71);
        assert_eq!(result.total_mult, 4);
        assert_eq!(result.final_score, 284);
    }

    #[test]
    fn test_high_card_scoring() {
        let cards = vec![c(Ace, Spades)];
        let levels = HandLevels::new();
        let result = calculate_score(&cards, &levels);

        assert_eq!(result.hand_type, PokerHand::HighCard);
        // Base: 5 chips, 1 mult
        // Ace scores: +11
        // Total: 16 * 1 = 16
        assert_eq!(result.total_chips, 16);
        assert_eq!(result.total_mult, 1);
        assert_eq!(result.final_score, 16);
    }

    #[test]
    fn test_leveled_hand() {
        let cards = vec![c(King, Spades), c(King, Hearts)];
        let mut levels = HandLevels::new();
        levels.level_up(PokerHand::Pair); // Level 2: 10+15=25 chips, 2+1=3 mult

        let result = calculate_score(&cards, &levels);

        assert_eq!(result.hand_type, PokerHand::Pair);
        // Base at level 2: 25 chips, 3 mult
        // Two kings: +10 +10 = 20
        // Total: 45 * 3 = 135
        assert_eq!(result.total_chips, 45);
        assert_eq!(result.total_mult, 3);
        assert_eq!(result.final_score, 135);
    }

    #[test]
    fn test_full_house_scoring() {
        let cards = vec![
            c(King, Spades),
            c(King, Hearts),
            c(King, Clubs),
            c(Five, Diamonds),
            c(Five, Spades),
        ];
        let levels = HandLevels::new();
        let result = calculate_score(&cards, &levels);

        assert_eq!(result.hand_type, PokerHand::FullHouse);
        // Base: 40 chips, 4 mult
        // All 5 score: 10+10+10+5+5 = 40
        // Total: 80 * 4 = 320
        assert_eq!(result.total_chips, 80);
        assert_eq!(result.total_mult, 4);
        assert_eq!(result.final_score, 320);
    }
}
