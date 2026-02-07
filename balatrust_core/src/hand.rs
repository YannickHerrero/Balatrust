use std::collections::HashMap;
use std::fmt;

use crate::card::{PlayingCard, Rank, Suit};

/// All recognized poker hands, ordered from worst to best
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PokerHand {
    HighCard,
    Pair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    RoyalFlush,
    FiveOfAKind,
    FlushHouse,
    FlushFive,
}

impl PokerHand {
    /// Base chips for this hand at level 1
    pub fn base_chips(&self) -> u64 {
        match self {
            PokerHand::HighCard => 5,
            PokerHand::Pair => 10,
            PokerHand::TwoPair => 20,
            PokerHand::ThreeOfAKind => 30,
            PokerHand::Straight => 30,
            PokerHand::Flush => 35,
            PokerHand::FullHouse => 40,
            PokerHand::FourOfAKind => 60,
            PokerHand::StraightFlush => 100,
            PokerHand::RoyalFlush => 100,
            PokerHand::FiveOfAKind => 120,
            PokerHand::FlushHouse => 140,
            PokerHand::FlushFive => 160,
        }
    }

    /// Base mult for this hand at level 1
    pub fn base_mult(&self) -> u64 {
        match self {
            PokerHand::HighCard => 1,
            PokerHand::Pair => 2,
            PokerHand::TwoPair => 2,
            PokerHand::ThreeOfAKind => 3,
            PokerHand::Straight => 4,
            PokerHand::Flush => 4,
            PokerHand::FullHouse => 4,
            PokerHand::FourOfAKind => 7,
            PokerHand::StraightFlush => 8,
            PokerHand::RoyalFlush => 8,
            PokerHand::FiveOfAKind => 12,
            PokerHand::FlushHouse => 14,
            PokerHand::FlushFive => 16,
        }
    }

    /// Chips gained per level up
    pub fn level_up_chips(&self) -> u64 {
        match self {
            PokerHand::HighCard => 10,
            PokerHand::Pair => 15,
            PokerHand::TwoPair => 20,
            PokerHand::ThreeOfAKind => 20,
            PokerHand::Straight => 30,
            PokerHand::Flush => 15,
            PokerHand::FullHouse => 25,
            PokerHand::FourOfAKind => 30,
            PokerHand::StraightFlush | PokerHand::RoyalFlush => 40,
            PokerHand::FiveOfAKind => 35,
            PokerHand::FlushHouse => 40,
            PokerHand::FlushFive => 50,
        }
    }

    /// Mult gained per level up
    pub fn level_up_mult(&self) -> u64 {
        match self {
            PokerHand::HighCard => 1,
            PokerHand::Pair => 1,
            PokerHand::TwoPair => 1,
            PokerHand::ThreeOfAKind => 2,
            PokerHand::Straight => 3,
            PokerHand::Flush => 2,
            PokerHand::FullHouse => 2,
            PokerHand::FourOfAKind => 3,
            PokerHand::StraightFlush | PokerHand::RoyalFlush => 4,
            PokerHand::FiveOfAKind => 3,
            PokerHand::FlushHouse => 4,
            PokerHand::FlushFive => 3,
        }
    }

    /// All poker hand variants
    pub const ALL: [PokerHand; 13] = [
        PokerHand::HighCard,
        PokerHand::Pair,
        PokerHand::TwoPair,
        PokerHand::ThreeOfAKind,
        PokerHand::Straight,
        PokerHand::Flush,
        PokerHand::FullHouse,
        PokerHand::FourOfAKind,
        PokerHand::StraightFlush,
        PokerHand::RoyalFlush,
        PokerHand::FiveOfAKind,
        PokerHand::FlushHouse,
        PokerHand::FlushFive,
    ];
}

impl fmt::Display for PokerHand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PokerHand::HighCard => "High Card",
            PokerHand::Pair => "Pair",
            PokerHand::TwoPair => "Two Pair",
            PokerHand::ThreeOfAKind => "Three of a Kind",
            PokerHand::Straight => "Straight",
            PokerHand::Flush => "Flush",
            PokerHand::FullHouse => "Full House",
            PokerHand::FourOfAKind => "Four of a Kind",
            PokerHand::StraightFlush => "Straight Flush",
            PokerHand::RoyalFlush => "Royal Flush",
            PokerHand::FiveOfAKind => "Five of a Kind",
            PokerHand::FlushHouse => "Flush House",
            PokerHand::FlushFive => "Flush Five",
        };
        write!(f, "{}", name)
    }
}

/// Result of detecting a poker hand from a set of played cards
#[derive(Debug, Clone)]
pub struct HandResult {
    pub hand_type: PokerHand,
    /// Indices into the played cards that are "scoring" (part of the hand)
    pub scoring_indices: Vec<usize>,
}

/// Detect the best poker hand from a set of played cards (up to 5).
/// Returns the hand type and which card indices contribute to scoring.
pub fn detect_hand(cards: &[PlayingCard]) -> HandResult {
    if cards.is_empty() {
        return HandResult {
            hand_type: PokerHand::HighCard,
            scoring_indices: vec![],
        };
    }

    let n = cards.len();

    // Build rank frequency map
    let mut rank_freq: HashMap<Rank, Vec<usize>> = HashMap::new();
    for (i, card) in cards.iter().enumerate() {
        rank_freq.entry(card.rank).or_default().push(i);
    }

    // Check flush (all same suit, considering wilds)
    let is_flush = n >= 5 && {
        // Try each suit - a card matches if it's that suit or is wild
        Suit::ALL
            .iter()
            .any(|&target_suit| cards.iter().all(|c| c.suit == target_suit || c.is_wild()))
    };

    // Check straight
    let is_straight = check_straight(cards);

    // Get groups sorted by size (descending), then rank (descending)
    let mut groups: Vec<(Rank, Vec<usize>)> = rank_freq.into_iter().collect();
    groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| b.0.cmp(&a.0)));

    let group_sizes: Vec<usize> = groups.iter().map(|(_, v)| v.len()).collect();

    // Check from best to worst hand
    // Flush Five: 5 of same rank + flush
    if n == 5 && group_sizes.first() == Some(&5) && is_flush {
        let all_indices: Vec<usize> = (0..n).collect();
        return HandResult {
            hand_type: PokerHand::FlushFive,
            scoring_indices: all_indices,
        };
    }

    // Five of a Kind: 5 of same rank
    if group_sizes.first() == Some(&5) {
        let all_indices: Vec<usize> = (0..n).collect();
        return HandResult {
            hand_type: PokerHand::FiveOfAKind,
            scoring_indices: all_indices,
        };
    }

    // Flush House: Full house + flush
    if n == 5 && group_sizes.len() == 2 && group_sizes[0] == 3 && group_sizes[1] == 2 && is_flush {
        let all_indices: Vec<usize> = (0..n).collect();
        return HandResult {
            hand_type: PokerHand::FlushHouse,
            scoring_indices: all_indices,
        };
    }

    // Royal Flush: A-K-Q-J-10 straight + flush
    if n == 5 && is_straight && is_flush && is_royal(cards) {
        let all_indices: Vec<usize> = (0..n).collect();
        return HandResult {
            hand_type: PokerHand::RoyalFlush,
            scoring_indices: all_indices,
        };
    }

    // Straight Flush: straight + flush
    if n == 5 && is_straight && is_flush {
        let all_indices: Vec<usize> = (0..n).collect();
        return HandResult {
            hand_type: PokerHand::StraightFlush,
            scoring_indices: all_indices,
        };
    }

    // Four of a Kind
    if group_sizes.first() >= Some(&4) {
        let scoring = groups[0].1.clone();
        return HandResult {
            hand_type: PokerHand::FourOfAKind,
            scoring_indices: scoring,
        };
    }

    // Full House: 3 + 2
    if group_sizes.len() >= 2 && group_sizes[0] == 3 && group_sizes[1] >= 2 {
        let mut scoring = groups[0].1.clone();
        scoring.extend_from_slice(&groups[1].1);
        return HandResult {
            hand_type: PokerHand::FullHouse,
            scoring_indices: scoring,
        };
    }

    // Flush
    if is_flush {
        let all_indices: Vec<usize> = (0..n).collect();
        return HandResult {
            hand_type: PokerHand::Flush,
            scoring_indices: all_indices,
        };
    }

    // Straight
    if is_straight {
        let all_indices: Vec<usize> = (0..n).collect();
        return HandResult {
            hand_type: PokerHand::Straight,
            scoring_indices: all_indices,
        };
    }

    // Three of a Kind
    if group_sizes.first() >= Some(&3) {
        let scoring = groups[0].1.clone();
        return HandResult {
            hand_type: PokerHand::ThreeOfAKind,
            scoring_indices: scoring,
        };
    }

    // Two Pair
    if group_sizes.len() >= 2 && group_sizes[0] >= 2 && group_sizes[1] >= 2 {
        let mut scoring = groups[0].1.clone();
        scoring.extend_from_slice(&groups[1].1);
        return HandResult {
            hand_type: PokerHand::TwoPair,
            scoring_indices: scoring,
        };
    }

    // Pair
    if group_sizes.first() >= Some(&2) {
        let scoring = groups[0].1.clone();
        return HandResult {
            hand_type: PokerHand::Pair,
            scoring_indices: scoring,
        };
    }

    // High Card: only the highest card scores
    let best_idx = cards
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| c.rank)
        .map(|(i, _)| i)
        .unwrap_or(0);

    HandResult {
        hand_type: PokerHand::HighCard,
        scoring_indices: vec![best_idx],
    }
}

/// Check if cards form a straight (5 consecutive ranks).
/// Handles Ace-low (A-2-3-4-5) and Ace-high (10-J-Q-K-A).
fn check_straight(cards: &[PlayingCard]) -> bool {
    if cards.len() != 5 {
        return false;
    }

    let mut ranks: Vec<u8> = cards.iter().map(|c| c.rank as u8).collect();
    ranks.sort();
    ranks.dedup();

    if ranks.len() != 5 {
        return false;
    }

    // Normal straight: max - min == 4
    if ranks[4] - ranks[0] == 4 {
        return true;
    }

    // Ace-low straight: A,2,3,4,5 => values 14,2,3,4,5
    if ranks == [2, 3, 4, 5, 14] {
        return true;
    }

    false
}

/// Check if cards form a royal (10-J-Q-K-A)
fn is_royal(cards: &[PlayingCard]) -> bool {
    let mut ranks: Vec<Rank> = cards.iter().map(|c| c.rank).collect();
    ranks.sort();
    ranks == [Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Rank::*;
    use crate::card::Suit::*;

    fn c(rank: Rank, suit: Suit) -> PlayingCard {
        PlayingCard::new(rank, suit)
    }

    #[test]
    fn test_high_card() {
        let cards = vec![c(Two, Spades), c(Five, Hearts), c(Nine, Clubs)];
        let result = detect_hand(&cards);
        assert_eq!(result.hand_type, PokerHand::HighCard);
        assert_eq!(result.scoring_indices.len(), 1);
    }

    #[test]
    fn test_pair() {
        let cards = vec![c(King, Spades), c(King, Hearts), c(Five, Clubs)];
        let result = detect_hand(&cards);
        assert_eq!(result.hand_type, PokerHand::Pair);
        assert_eq!(result.scoring_indices.len(), 2);
    }

    #[test]
    fn test_two_pair() {
        let cards = vec![
            c(King, Spades),
            c(King, Hearts),
            c(Five, Clubs),
            c(Five, Diamonds),
            c(Three, Spades),
        ];
        let result = detect_hand(&cards);
        assert_eq!(result.hand_type, PokerHand::TwoPair);
        assert_eq!(result.scoring_indices.len(), 4);
    }

    #[test]
    fn test_three_of_a_kind() {
        let cards = vec![c(Queen, Spades), c(Queen, Hearts), c(Queen, Clubs)];
        let result = detect_hand(&cards);
        assert_eq!(result.hand_type, PokerHand::ThreeOfAKind);
        assert_eq!(result.scoring_indices.len(), 3);
    }

    #[test]
    fn test_straight() {
        let cards = vec![
            c(Five, Spades),
            c(Six, Hearts),
            c(Seven, Clubs),
            c(Eight, Diamonds),
            c(Nine, Spades),
        ];
        let result = detect_hand(&cards);
        assert_eq!(result.hand_type, PokerHand::Straight);
        assert_eq!(result.scoring_indices.len(), 5);
    }

    #[test]
    fn test_ace_low_straight() {
        let cards = vec![
            c(Ace, Spades),
            c(Two, Hearts),
            c(Three, Clubs),
            c(Four, Diamonds),
            c(Five, Spades),
        ];
        let result = detect_hand(&cards);
        assert_eq!(result.hand_type, PokerHand::Straight);
    }

    #[test]
    fn test_flush() {
        let cards = vec![
            c(Two, Hearts),
            c(Five, Hearts),
            c(Eight, Hearts),
            c(Jack, Hearts),
            c(Ace, Hearts),
        ];
        let result = detect_hand(&cards);
        assert_eq!(result.hand_type, PokerHand::Flush);
        assert_eq!(result.scoring_indices.len(), 5);
    }

    #[test]
    fn test_full_house() {
        let cards = vec![
            c(King, Spades),
            c(King, Hearts),
            c(King, Clubs),
            c(Five, Diamonds),
            c(Five, Spades),
        ];
        let result = detect_hand(&cards);
        assert_eq!(result.hand_type, PokerHand::FullHouse);
        assert_eq!(result.scoring_indices.len(), 5);
    }

    #[test]
    fn test_four_of_a_kind() {
        let cards = vec![
            c(Ace, Spades),
            c(Ace, Hearts),
            c(Ace, Clubs),
            c(Ace, Diamonds),
            c(Three, Spades),
        ];
        let result = detect_hand(&cards);
        assert_eq!(result.hand_type, PokerHand::FourOfAKind);
        assert_eq!(result.scoring_indices.len(), 4);
    }

    #[test]
    fn test_straight_flush() {
        let cards = vec![
            c(Five, Hearts),
            c(Six, Hearts),
            c(Seven, Hearts),
            c(Eight, Hearts),
            c(Nine, Hearts),
        ];
        let result = detect_hand(&cards);
        assert_eq!(result.hand_type, PokerHand::StraightFlush);
    }

    #[test]
    fn test_royal_flush() {
        let cards = vec![
            c(Ten, Spades),
            c(Jack, Spades),
            c(Queen, Spades),
            c(King, Spades),
            c(Ace, Spades),
        ];
        let result = detect_hand(&cards);
        assert_eq!(result.hand_type, PokerHand::RoyalFlush);
    }

    #[test]
    fn test_five_of_a_kind() {
        // This requires modified cards (e.g. stone cards or wilds)
        // For now test with 5 same rank
        let cards = vec![
            c(King, Spades),
            c(King, Hearts),
            c(King, Clubs),
            c(King, Diamonds),
            c(King, Spades), // Duplicate for test
        ];
        let result = detect_hand(&cards);
        assert_eq!(result.hand_type, PokerHand::FiveOfAKind);
    }
}
