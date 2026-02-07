use std::fmt;

/// The type of blind within an ante
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlindType {
    Small,
    Big,
    Boss(BossBlind),
}

impl BlindType {
    pub fn name(&self) -> String {
        match self {
            BlindType::Small => "Small Blind".to_string(),
            BlindType::Big => "Big Blind".to_string(),
            BlindType::Boss(boss) => format!("{}", boss),
        }
    }

    /// Score multiplier applied to the ante's base requirement
    pub fn score_multiplier(&self) -> f64 {
        match self {
            BlindType::Small => 1.0,
            BlindType::Big => 1.5,
            BlindType::Boss(boss) => boss.score_multiplier(),
        }
    }

    /// Money reward for beating this blind
    pub fn reward(&self) -> u32 {
        match self {
            BlindType::Small => 3,
            BlindType::Big => 4,
            BlindType::Boss(_) => 5,
        }
    }

    /// Can this blind be skipped?
    pub fn can_skip(&self) -> bool {
        matches!(self, BlindType::Small | BlindType::Big)
    }
}

impl fmt::Display for BlindType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Boss blind effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBlind {
    /// Discards 2 random cards per hand played
    TheHook,
    /// 4x base chips (extra large target)
    TheWall,
    /// Must play exactly 5 cards
    ThePsychic,
    /// Only 1 hand allowed
    TheNeedle,
    /// All Club cards debuffed
    TheClub,
    /// All Spade cards debuffed
    TheGoad,
    /// All Diamond cards debuffed
    TheWindow,
    /// All Heart cards debuffed
    TheHead,
}

impl BossBlind {
    pub const ALL: [BossBlind; 8] = [
        BossBlind::TheHook,
        BossBlind::TheWall,
        BossBlind::ThePsychic,
        BossBlind::TheNeedle,
        BossBlind::TheClub,
        BossBlind::TheGoad,
        BossBlind::TheWindow,
        BossBlind::TheHead,
    ];

    pub fn score_multiplier(&self) -> f64 {
        match self {
            BossBlind::TheWall => 4.0,
            _ => 2.0,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BossBlind::TheHook => "Discards 2 random cards per hand",
            BossBlind::TheWall => "Extra large blind (4x chips)",
            BossBlind::ThePsychic => "Must play exactly 5 cards",
            BossBlind::TheNeedle => "Only 1 hand allowed",
            BossBlind::TheClub => "All Club cards are debuffed",
            BossBlind::TheGoad => "All Spade cards are debuffed",
            BossBlind::TheWindow => "All Diamond cards are debuffed",
            BossBlind::TheHead => "All Heart cards are debuffed",
        }
    }
}

impl fmt::Display for BossBlind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            BossBlind::TheHook => "The Hook",
            BossBlind::TheWall => "The Wall",
            BossBlind::ThePsychic => "The Psychic",
            BossBlind::TheNeedle => "The Needle",
            BossBlind::TheClub => "The Club",
            BossBlind::TheGoad => "The Goad",
            BossBlind::TheWindow => "The Window",
            BossBlind::TheHead => "The Head",
        };
        write!(f, "{}", name)
    }
}

/// Base chip requirements per ante
pub fn ante_base_chips(ante: u8) -> u64 {
    match ante {
        1 => 300,
        2 => 800,
        3 => 2_000,
        4 => 5_000,
        5 => 11_000,
        6 => 20_000,
        7 => 35_000,
        8 => 50_000,
        _ => 50_000 + (ante as u64 - 8) * 25_000, // Endless scaling
    }
}

/// Calculate the score target for a given ante and blind type
pub fn score_target(ante: u8, blind_type: &BlindType) -> u64 {
    let base = ante_base_chips(ante) as f64;
    (base * blind_type.score_multiplier()) as u64
}
