use std::fmt;

use crate::hand::PokerHand;

/// Type of consumable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsumableType {
    Planet(PlanetCard),
    Tarot(TarotCard),
}

impl ConsumableType {
    pub fn name(&self) -> &'static str {
        match self {
            ConsumableType::Planet(p) => p.name(),
            ConsumableType::Tarot(t) => t.name(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            ConsumableType::Planet(p) => p.description(),
            ConsumableType::Tarot(t) => t.description().to_string(),
        }
    }

    pub fn price(&self) -> u32 {
        match self {
            ConsumableType::Planet(_) => 3,
            ConsumableType::Tarot(_) => 3,
        }
    }
}

impl fmt::Display for ConsumableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A consumable instance
#[derive(Debug, Clone)]
pub struct Consumable {
    pub consumable_type: ConsumableType,
}

impl Consumable {
    pub fn new(consumable_type: ConsumableType) -> Self {
        Self { consumable_type }
    }

    pub fn planet(card: PlanetCard) -> Self {
        Self::new(ConsumableType::Planet(card))
    }

    pub fn tarot(card: TarotCard) -> Self {
        Self::new(ConsumableType::Tarot(card))
    }
}

/// Planet cards level up specific poker hands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlanetCard {
    Pluto,   // High Card
    Mercury, // Pair
    Uranus,  // Two Pair
    Venus,   // Three of a Kind
    Saturn,  // Straight
    Jupiter, // Flush
    Earth,   // Full House
    Mars,    // Four of a Kind
    Neptune, // Straight Flush
    PlanetX, // Five of a Kind
    Ceres,   // Flush House
    Eris,    // Flush Five
}

impl PlanetCard {
    pub const ALL: [PlanetCard; 12] = [
        PlanetCard::Pluto,
        PlanetCard::Mercury,
        PlanetCard::Uranus,
        PlanetCard::Venus,
        PlanetCard::Saturn,
        PlanetCard::Jupiter,
        PlanetCard::Earth,
        PlanetCard::Mars,
        PlanetCard::Neptune,
        PlanetCard::PlanetX,
        PlanetCard::Ceres,
        PlanetCard::Eris,
    ];

    /// Common planet cards (not including secret hand planets)
    pub const COMMON: [PlanetCard; 9] = [
        PlanetCard::Pluto,
        PlanetCard::Mercury,
        PlanetCard::Uranus,
        PlanetCard::Venus,
        PlanetCard::Saturn,
        PlanetCard::Jupiter,
        PlanetCard::Earth,
        PlanetCard::Mars,
        PlanetCard::Neptune,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            PlanetCard::Pluto => "Pluto",
            PlanetCard::Mercury => "Mercury",
            PlanetCard::Uranus => "Uranus",
            PlanetCard::Venus => "Venus",
            PlanetCard::Saturn => "Saturn",
            PlanetCard::Jupiter => "Jupiter",
            PlanetCard::Earth => "Earth",
            PlanetCard::Mars => "Mars",
            PlanetCard::Neptune => "Neptune",
            PlanetCard::PlanetX => "Planet X",
            PlanetCard::Ceres => "Ceres",
            PlanetCard::Eris => "Eris",
        }
    }

    pub fn hand_type(&self) -> PokerHand {
        match self {
            PlanetCard::Pluto => PokerHand::HighCard,
            PlanetCard::Mercury => PokerHand::Pair,
            PlanetCard::Uranus => PokerHand::TwoPair,
            PlanetCard::Venus => PokerHand::ThreeOfAKind,
            PlanetCard::Saturn => PokerHand::Straight,
            PlanetCard::Jupiter => PokerHand::Flush,
            PlanetCard::Earth => PokerHand::FullHouse,
            PlanetCard::Mars => PokerHand::FourOfAKind,
            PlanetCard::Neptune => PokerHand::StraightFlush,
            PlanetCard::PlanetX => PokerHand::FiveOfAKind,
            PlanetCard::Ceres => PokerHand::FlushHouse,
            PlanetCard::Eris => PokerHand::FlushFive,
        }
    }

    pub fn description(&self) -> String {
        let hand = self.hand_type();
        format!(
            "Level up {} (+{} Chips, +{} Mult)",
            hand,
            hand.level_up_chips(),
            hand.level_up_mult()
        )
    }
}

/// Tarot cards modify playing cards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TarotCard {
    TheFool,          // Create copy of last Tarot/Planet used
    TheMagician,      // Enhance 1-2 cards to Lucky
    TheHighPriestess, // Create up to 2 Planet cards
    TheEmpress,       // Enhance 1-2 cards to Mult
    TheEmperor,       // Create up to 2 Tarot cards
    TheHierophant,    // Enhance 1-2 cards to Bonus
    TheLover,         // Enhance 1 card to Wild
    TheChariot,       // Enhance 1 card to Steel
    Strength,         // Increase rank of up to 2 cards by 1
    TheHermit,        // Double money (max $20)
    Death,            // Convert left card to right card (2 selected)
    Temperance,       // Gain money equal to sell value of jokers (max $50)
}

impl TarotCard {
    pub const ALL: [TarotCard; 12] = [
        TarotCard::TheFool,
        TarotCard::TheMagician,
        TarotCard::TheHighPriestess,
        TarotCard::TheEmpress,
        TarotCard::TheEmperor,
        TarotCard::TheHierophant,
        TarotCard::TheLover,
        TarotCard::TheChariot,
        TarotCard::Strength,
        TarotCard::TheHermit,
        TarotCard::Death,
        TarotCard::Temperance,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            TarotCard::TheFool => "The Fool",
            TarotCard::TheMagician => "The Magician",
            TarotCard::TheHighPriestess => "High Priestess",
            TarotCard::TheEmpress => "The Empress",
            TarotCard::TheEmperor => "The Emperor",
            TarotCard::TheHierophant => "Hierophant",
            TarotCard::TheLover => "The Lover",
            TarotCard::TheChariot => "The Chariot",
            TarotCard::Strength => "Strength",
            TarotCard::TheHermit => "The Hermit",
            TarotCard::Death => "Death",
            TarotCard::Temperance => "Temperance",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TarotCard::TheFool => "Copy last Tarot/Planet used",
            TarotCard::TheMagician => "Enhance 1-2 cards to Lucky",
            TarotCard::TheHighPriestess => "Create up to 2 Planet cards",
            TarotCard::TheEmpress => "Enhance 1-2 cards to Mult",
            TarotCard::TheEmperor => "Create up to 2 Tarot cards",
            TarotCard::TheHierophant => "Enhance 1-2 cards to Bonus",
            TarotCard::TheLover => "Enhance 1 card to Wild",
            TarotCard::TheChariot => "Enhance 1 card to Steel",
            TarotCard::Strength => "Increase rank of 1-2 cards by 1",
            TarotCard::TheHermit => "Double money (max $20)",
            TarotCard::Death => "Convert left card to right card",
            TarotCard::Temperance => "Gain $ equal to joker sell value",
        }
    }

    /// How many cards this tarot needs selected
    pub fn cards_needed(&self) -> (usize, usize) {
        match self {
            TarotCard::TheFool => (0, 0),
            TarotCard::TheHighPriestess => (0, 0),
            TarotCard::TheEmperor => (0, 0),
            TarotCard::TheHermit => (0, 0),
            TarotCard::Temperance => (0, 0),
            TarotCard::TheLover | TarotCard::TheChariot => (1, 1),
            TarotCard::Death => (2, 2),
            _ => (1, 2), // 1-2 cards
        }
    }
}
