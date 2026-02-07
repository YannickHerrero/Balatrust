use std::fmt;

use crate::card::{PlayingCard, Rank, Suit};
use crate::hand::PokerHand;

/// Rarity tier for jokers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JokerRarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl JokerRarity {
    pub fn base_price(&self) -> u32 {
        match self {
            JokerRarity::Common => 4,
            JokerRarity::Uncommon => 6,
            JokerRarity::Rare => 8,
            JokerRarity::Legendary => 20,
        }
    }
}

/// All joker types in the MVP
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JokerType {
    // Flat mult
    Joker, // +4 Mult
    // Suit conditional mult
    GreedyJoker,     // +3 Mult per Diamond scored
    LustyJoker,      // +3 Mult per Heart scored
    WrathfulJoker,   // +3 Mult per Spade scored
    GluttonousJoker, // +3 Mult per Club scored
    // Hand conditional mult
    JollyJoker, // +8 Mult if hand contains Pair
    ZanyJoker,  // +12 Mult if hand contains Three of a Kind
    CrazyJoker, // +12 Mult if hand contains Straight
    // Size conditional
    HalfJoker, // +20 Mult if played hand <=3 cards
    // Chips
    Banner,  // +30 Chips per discard remaining
    OddTodd, // +31 Chips per odd-ranked card scored
    Scholar, // +20 Chips and +4 Mult per Ace scored
    // xMult
    SteelJoker, // xMult based on steel cards in hand
    Blackboard, // x3 Mult if all held cards are Spades or Clubs
    TheDuo,     // x2 Mult if hand contains Pair
    // Economy
    Egg,         // +$3 sell value per round
    GoldenJoker, // +$4 at end of round
    // Retrigger
    Hack, // Retrigger 2,3,4,5 cards
    // Meta
    Blueprint, // Copy joker to the right
    // Hand conditional xMult
    TheTrio, // x3 Mult if hand contains Three of a Kind
}

impl JokerType {
    pub const ALL: [JokerType; 20] = [
        JokerType::Joker,
        JokerType::GreedyJoker,
        JokerType::LustyJoker,
        JokerType::WrathfulJoker,
        JokerType::GluttonousJoker,
        JokerType::JollyJoker,
        JokerType::ZanyJoker,
        JokerType::CrazyJoker,
        JokerType::HalfJoker,
        JokerType::Banner,
        JokerType::OddTodd,
        JokerType::Scholar,
        JokerType::SteelJoker,
        JokerType::Blackboard,
        JokerType::TheDuo,
        JokerType::Egg,
        JokerType::GoldenJoker,
        JokerType::Hack,
        JokerType::Blueprint,
        JokerType::TheTrio,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            JokerType::Joker => "Joker",
            JokerType::GreedyJoker => "Greedy Joker",
            JokerType::LustyJoker => "Lusty Joker",
            JokerType::WrathfulJoker => "Wrathful Joker",
            JokerType::GluttonousJoker => "Gluttonous Joker",
            JokerType::JollyJoker => "Jolly Joker",
            JokerType::ZanyJoker => "Zany Joker",
            JokerType::CrazyJoker => "Crazy Joker",
            JokerType::HalfJoker => "Half Joker",
            JokerType::Banner => "Banner",
            JokerType::OddTodd => "Odd Todd",
            JokerType::Scholar => "Scholar",
            JokerType::SteelJoker => "Steel Joker",
            JokerType::Blackboard => "Blackboard",
            JokerType::TheDuo => "The Duo",
            JokerType::Egg => "Egg",
            JokerType::GoldenJoker => "Golden Joker",
            JokerType::Hack => "Hack",
            JokerType::Blueprint => "Blueprint",
            JokerType::TheTrio => "The Trio",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            JokerType::Joker => "+4 Mult",
            JokerType::GreedyJoker => "+3 Mult per Diamond",
            JokerType::LustyJoker => "+3 Mult per Heart",
            JokerType::WrathfulJoker => "+3 Mult per Spade",
            JokerType::GluttonousJoker => "+3 Mult per Club",
            JokerType::JollyJoker => "+8 Mult if Pair in hand",
            JokerType::ZanyJoker => "+12 Mult if Three of a Kind",
            JokerType::CrazyJoker => "+12 Mult if Straight",
            JokerType::HalfJoker => "+20 Mult if <=3 cards",
            JokerType::Banner => "+30 Chips per discard left",
            JokerType::OddTodd => "+31 Chips per odd card",
            JokerType::Scholar => "+20 Chips, +4 Mult per Ace",
            JokerType::SteelJoker => "x0.2 Mult per Steel card",
            JokerType::Blackboard => "x3 if held cards all dark",
            JokerType::TheDuo => "x2 Mult if Pair in hand",
            JokerType::Egg => "+$3 sell value per round",
            JokerType::GoldenJoker => "+$4 at end of round",
            JokerType::Hack => "Retrigger 2,3,4,5 cards",
            JokerType::Blueprint => "Copy joker to the right",
            JokerType::TheTrio => "x3 if Three of a Kind",
        }
    }

    pub fn rarity(&self) -> JokerRarity {
        match self {
            JokerType::Joker
            | JokerType::GreedyJoker
            | JokerType::LustyJoker
            | JokerType::WrathfulJoker
            | JokerType::GluttonousJoker
            | JokerType::JollyJoker
            | JokerType::ZanyJoker
            | JokerType::CrazyJoker
            | JokerType::HalfJoker
            | JokerType::Banner
            | JokerType::OddTodd
            | JokerType::Egg
            | JokerType::GoldenJoker
            | JokerType::Hack => JokerRarity::Common,
            JokerType::Scholar | JokerType::SteelJoker | JokerType::TheDuo | JokerType::TheTrio => {
                JokerRarity::Uncommon
            }
            JokerType::Blackboard | JokerType::Blueprint => JokerRarity::Rare,
        }
    }

    pub fn price(&self) -> u32 {
        self.rarity().base_price()
    }
}

impl fmt::Display for JokerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A joker instance owned by the player
#[derive(Debug, Clone)]
pub struct Joker {
    pub joker_type: JokerType,
    pub sell_value: u32,
    /// For Egg: accumulates bonus sell value
    pub bonus_sell: u32,
}

impl Joker {
    pub fn new(joker_type: JokerType) -> Self {
        Self {
            sell_value: joker_type.price() / 2,
            bonus_sell: 0,
            joker_type,
        }
    }

    pub fn total_sell_value(&self) -> u32 {
        self.sell_value + self.bonus_sell
    }
}

/// Context needed for joker effect evaluation
pub struct JokerContext<'a> {
    pub played_cards: &'a [PlayingCard],
    pub scoring_indices: &'a [usize],
    pub hand_type: PokerHand,
    pub held_cards: &'a [PlayingCard],
    pub discards_remaining: u8,
    pub num_played: usize,
}

/// The effect a joker applies to scoring
#[derive(Debug, Clone)]
pub enum JokerEffect {
    /// Add flat chips
    AddChips(u64),
    /// Add flat mult
    AddMult(u64),
    /// Multiply mult
    XMult(f64),
    /// Add chips for specific scoring card indices
    AddChipsPerCard {
        card_indices: Vec<usize>,
        chips_each: u64,
    },
    /// Add mult for specific scoring card indices
    AddMultPerCard {
        card_indices: Vec<usize>,
        mult_each: u64,
    },
    /// Add both chips and mult for specific cards
    AddChipsAndMultPerCard {
        card_indices: Vec<usize>,
        chips_each: u64,
        mult_each: u64,
    },
    /// Retrigger specific scoring cards
    Retrigger { card_indices: Vec<usize> },
    /// No scoring effect (economy jokers)
    None,
}

/// Evaluate a joker's effect given the current context.
/// For Blueprint, pass the next joker's type to copy.
pub fn evaluate_joker(
    joker: &Joker,
    ctx: &JokerContext,
    next_joker_type: Option<JokerType>,
) -> JokerEffect {
    let jtype = if joker.joker_type == JokerType::Blueprint {
        match next_joker_type {
            Some(t) => t,
            None => return JokerEffect::None,
        }
    } else {
        joker.joker_type
    };

    evaluate_type(jtype, ctx)
}

fn evaluate_type(jtype: JokerType, ctx: &JokerContext) -> JokerEffect {
    match jtype {
        JokerType::Joker => JokerEffect::AddMult(4),

        JokerType::GreedyJoker => suit_mult_joker(ctx, Suit::Diamonds, 3),
        JokerType::LustyJoker => suit_mult_joker(ctx, Suit::Hearts, 3),
        JokerType::WrathfulJoker => suit_mult_joker(ctx, Suit::Spades, 3),
        JokerType::GluttonousJoker => suit_mult_joker(ctx, Suit::Clubs, 3),

        JokerType::JollyJoker => {
            if ctx.hand_type >= PokerHand::Pair {
                JokerEffect::AddMult(8)
            } else {
                JokerEffect::None
            }
        }
        JokerType::ZanyJoker => {
            if ctx.hand_type >= PokerHand::ThreeOfAKind {
                JokerEffect::AddMult(12)
            } else {
                JokerEffect::None
            }
        }
        JokerType::CrazyJoker => {
            if ctx.hand_type >= PokerHand::Straight
                || ctx.hand_type == PokerHand::StraightFlush
                || ctx.hand_type == PokerHand::RoyalFlush
            {
                JokerEffect::AddMult(12)
            } else {
                JokerEffect::None
            }
        }

        JokerType::HalfJoker => {
            if ctx.num_played <= 3 {
                JokerEffect::AddMult(20)
            } else {
                JokerEffect::None
            }
        }

        JokerType::Banner => JokerEffect::AddChips(ctx.discards_remaining as u64 * 30),

        JokerType::OddTodd => {
            let odd_indices: Vec<usize> = ctx
                .scoring_indices
                .iter()
                .filter(|&&i| {
                    let rank_val = ctx.played_cards[i].rank as u8;
                    rank_val % 2 == 1 // Odd ranks: 3,5,7,9,J(11),K(13)
                })
                .copied()
                .collect();
            if odd_indices.is_empty() {
                JokerEffect::None
            } else {
                JokerEffect::AddChipsPerCard {
                    card_indices: odd_indices,
                    chips_each: 31,
                }
            }
        }

        JokerType::Scholar => {
            let ace_indices: Vec<usize> = ctx
                .scoring_indices
                .iter()
                .filter(|&&i| ctx.played_cards[i].rank == Rank::Ace)
                .copied()
                .collect();
            if ace_indices.is_empty() {
                JokerEffect::None
            } else {
                JokerEffect::AddChipsAndMultPerCard {
                    card_indices: ace_indices,
                    chips_each: 20,
                    mult_each: 4,
                }
            }
        }

        JokerType::SteelJoker => {
            let steel_count = ctx
                .held_cards
                .iter()
                .filter(|c| c.enhancement == Some(crate::card::Enhancement::Steel))
                .count();
            if steel_count > 0 {
                JokerEffect::XMult(1.0 + 0.2 * steel_count as f64)
            } else {
                JokerEffect::None
            }
        }

        JokerType::Blackboard => {
            let all_dark = ctx
                .held_cards
                .iter()
                .all(|c| c.suit == Suit::Spades || c.suit == Suit::Clubs || c.is_wild());
            if all_dark && !ctx.held_cards.is_empty() {
                JokerEffect::XMult(3.0)
            } else {
                JokerEffect::None
            }
        }

        JokerType::TheDuo => {
            if hand_contains_at_least(ctx, PokerHand::Pair) {
                JokerEffect::XMult(2.0)
            } else {
                JokerEffect::None
            }
        }

        JokerType::TheTrio => {
            if hand_contains_at_least(ctx, PokerHand::ThreeOfAKind) {
                JokerEffect::XMult(3.0)
            } else {
                JokerEffect::None
            }
        }

        JokerType::Egg | JokerType::GoldenJoker => JokerEffect::None,

        JokerType::Hack => {
            let retrigger_indices: Vec<usize> = ctx
                .scoring_indices
                .iter()
                .filter(|&&i| {
                    let rank = ctx.played_cards[i].rank;
                    matches!(rank, Rank::Two | Rank::Three | Rank::Four | Rank::Five)
                })
                .copied()
                .collect();
            if retrigger_indices.is_empty() {
                JokerEffect::None
            } else {
                JokerEffect::Retrigger {
                    card_indices: retrigger_indices,
                }
            }
        }

        JokerType::Blueprint => {
            // Handled in evaluate_joker
            JokerEffect::None
        }
    }
}

fn suit_mult_joker(ctx: &JokerContext, suit: Suit, mult_per: u64) -> JokerEffect {
    let matching: Vec<usize> = ctx
        .scoring_indices
        .iter()
        .filter(|&&i| {
            let card = &ctx.played_cards[i];
            card.suit == suit || card.is_wild()
        })
        .copied()
        .collect();

    if matching.is_empty() {
        JokerEffect::None
    } else {
        JokerEffect::AddMultPerCard {
            card_indices: matching,
            mult_each: mult_per,
        }
    }
}

fn hand_contains_at_least(ctx: &JokerContext, min_hand: PokerHand) -> bool {
    ctx.hand_type >= min_hand
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::PlayingCard;
    use crate::card::Rank::*;
    use crate::card::Suit::*;

    fn c(rank: Rank, suit: Suit) -> PlayingCard {
        PlayingCard::new(rank, suit)
    }

    #[test]
    fn test_joker_basic_mult() {
        let joker = Joker::new(JokerType::Joker);
        let cards = vec![c(Ace, Spades), c(King, Hearts)];
        let ctx = JokerContext {
            played_cards: &cards,
            scoring_indices: &[0, 1],
            hand_type: PokerHand::HighCard,
            held_cards: &[],
            discards_remaining: 3,
            num_played: 2,
        };
        let effect = evaluate_joker(&joker, &ctx, None);
        match effect {
            JokerEffect::AddMult(m) => assert_eq!(m, 4),
            _ => panic!("Expected AddMult"),
        }
    }

    #[test]
    fn test_greedy_joker_diamonds() {
        let joker = Joker::new(JokerType::GreedyJoker);
        let cards = vec![c(Ace, Diamonds), c(King, Diamonds), c(Five, Hearts)];
        let ctx = JokerContext {
            played_cards: &cards,
            scoring_indices: &[0, 1, 2],
            hand_type: PokerHand::HighCard,
            held_cards: &[],
            discards_remaining: 3,
            num_played: 3,
        };
        let effect = evaluate_joker(&joker, &ctx, None);
        match effect {
            JokerEffect::AddMultPerCard {
                card_indices,
                mult_each,
            } => {
                assert_eq!(card_indices.len(), 2); // Two diamonds
                assert_eq!(mult_each, 3);
            }
            _ => panic!("Expected AddMultPerCard"),
        }
    }

    #[test]
    fn test_the_duo_x_mult() {
        let joker = Joker::new(JokerType::TheDuo);
        let cards = vec![c(King, Spades), c(King, Hearts)];
        let ctx = JokerContext {
            played_cards: &cards,
            scoring_indices: &[0, 1],
            hand_type: PokerHand::Pair,
            held_cards: &[],
            discards_remaining: 3,
            num_played: 2,
        };
        let effect = evaluate_joker(&joker, &ctx, None);
        match effect {
            JokerEffect::XMult(x) => assert!((x - 2.0).abs() < f64::EPSILON),
            _ => panic!("Expected XMult"),
        }
    }

    #[test]
    fn test_banner_chips_per_discard() {
        let joker = Joker::new(JokerType::Banner);
        let cards = vec![c(Ace, Spades)];
        let ctx = JokerContext {
            played_cards: &cards,
            scoring_indices: &[0],
            hand_type: PokerHand::HighCard,
            held_cards: &[],
            discards_remaining: 3,
            num_played: 1,
        };
        let effect = evaluate_joker(&joker, &ctx, None);
        match effect {
            JokerEffect::AddChips(c) => assert_eq!(c, 90), // 3 * 30
            _ => panic!("Expected AddChips"),
        }
    }

    #[test]
    fn test_half_joker_small_hand() {
        let joker = Joker::new(JokerType::HalfJoker);
        let cards = vec![c(Ace, Spades), c(King, Hearts)];
        let ctx = JokerContext {
            played_cards: &cards,
            scoring_indices: &[0, 1],
            hand_type: PokerHand::HighCard,
            held_cards: &[],
            discards_remaining: 3,
            num_played: 2,
        };
        let effect = evaluate_joker(&joker, &ctx, None);
        match effect {
            JokerEffect::AddMult(m) => assert_eq!(m, 20),
            _ => panic!("Expected AddMult"),
        }
    }
}
