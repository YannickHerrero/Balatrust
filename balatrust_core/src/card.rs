use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

    pub fn symbol(&self) -> char {
        match self {
            Suit::Spades => '\u{2660}',   // ♠
            Suit::Hearts => '\u{2665}',   // ♥
            Suit::Diamonds => '\u{2666}', // ♦
            Suit::Clubs => '\u{2663}',    // ♣
        }
    }

    pub fn is_red(&self) -> bool {
        matches!(self, Suit::Hearts | Suit::Diamonds)
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

impl Rank {
    pub const ALL: [Rank; 13] = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];

    /// Chip value when this card scores
    pub fn chip_value(&self) -> u64 {
        match self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
            Rank::Ace => 11,
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ace => "A",
        }
    }

    pub fn is_face(&self) -> bool {
        matches!(self, Rank::Jack | Rank::Queen | Rank::King)
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Enhancement {
    Bonus, // +30 chips
    Mult,  // +4 mult
    Wild,  // Counts as all suits
    Glass, // x2 mult, 1/4 chance to destroy
    Steel, // x1.5 mult while in hand
    Stone, // +50 chips, no rank/suit, always scores
    Gold,  // $3 if held at end of round
    Lucky, // 1/5 +20 mult, 1/15 $20
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edition {
    Base,
    Foil,        // +50 chips
    Holographic, // +10 mult
    Polychrome,  // x1.5 mult
}

impl Default for Edition {
    fn default() -> Self {
        Edition::Base
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Seal {
    Gold,   // $3 when played
    Red,    // Retrigger 1x
    Blue,   // Creates planet card
    Purple, // Creates tarot when discarded
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayingCard {
    pub rank: Rank,
    pub suit: Suit,
    pub enhancement: Option<Enhancement>,
    pub edition: Edition,
    pub seal: Option<Seal>,
    pub debuffed: bool,
}

impl PlayingCard {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self {
            rank,
            suit,
            enhancement: None,
            edition: Edition::default(),
            seal: None,
            debuffed: false,
        }
    }

    /// Effective chip value considering enhancements
    pub fn chip_value(&self) -> u64 {
        if self.debuffed {
            return 0;
        }
        let base = match self.enhancement {
            Some(Enhancement::Stone) => 50,
            _ => self.rank.chip_value(),
        };
        let bonus = match self.enhancement {
            Some(Enhancement::Bonus) => 30,
            _ => 0,
        };
        let edition_bonus = match self.edition {
            Edition::Foil => 50,
            _ => 0,
        };
        base + bonus + edition_bonus
    }

    /// Additional mult from this card's enhancement/edition
    pub fn mult_bonus(&self) -> u64 {
        if self.debuffed {
            return 0;
        }
        let enh = match self.enhancement {
            Some(Enhancement::Mult) => 4,
            _ => 0,
        };
        let ed = match self.edition {
            Edition::Holographic => 10,
            _ => 0,
        };
        enh + ed
    }

    /// Multiplicative mult from this card
    pub fn x_mult(&self) -> f64 {
        if self.debuffed {
            return 1.0;
        }
        let enh = match self.enhancement {
            Some(Enhancement::Glass) => 2.0,
            _ => 1.0,
        };
        let ed = match self.edition {
            Edition::Polychrome => 1.5,
            _ => 1.0,
        };
        enh * ed
    }

    /// Whether this card acts as a wild (all suits)
    pub fn is_wild(&self) -> bool {
        matches!(self.enhancement, Some(Enhancement::Wild))
    }

    /// Whether this card always scores regardless of hand
    pub fn always_scores(&self) -> bool {
        matches!(self.enhancement, Some(Enhancement::Stone))
    }
}

impl fmt::Display for PlayingCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}
