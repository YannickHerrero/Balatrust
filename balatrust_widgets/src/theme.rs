use ratatui::style::Color;

/// Balatro-inspired color theme for the TUI
pub struct Theme;

impl Theme {
    // Backgrounds
    pub const BG: Color = Color::Rgb(13, 17, 23);
    pub const FELT: Color = Color::Rgb(22, 22, 52);
    pub const PANEL_BG: Color = Color::Rgb(30, 30, 60);

    // Card colors
    pub const CARD_FACE: Color = Color::Rgb(240, 240, 240);
    pub const CARD_BORDER: Color = Color::Rgb(108, 117, 125);
    pub const CARD_SELECTED: Color = Color::Rgb(255, 214, 10);
    pub const CARD_BACK: Color = Color::Rgb(60, 60, 120);
    /// Subtle dim color used for decorative interior elements on card faces
    pub const CARD_FACE_DIM: Color = Color::Rgb(80, 80, 100);

    // Suit colors
    pub const RED_SUIT: Color = Color::Rgb(230, 57, 70);
    pub const BLACK_SUIT: Color = Color::Rgb(224, 224, 224);

    // Score colors
    pub const CHIPS_COLOR: Color = Color::Rgb(76, 201, 240);
    pub const MULT_COLOR: Color = Color::Rgb(230, 57, 70);
    pub const XMULT_COLOR: Color = Color::Rgb(255, 100, 100);
    pub const SCORE_COLOR: Color = Color::Rgb(255, 214, 10);

    // Money
    pub const MONEY_COLOR: Color = Color::Rgb(6, 214, 160);

    // Joker rarities
    pub const COMMON: Color = Color::Rgb(108, 117, 125);
    pub const UNCOMMON: Color = Color::Rgb(6, 214, 160);
    pub const RARE: Color = Color::Rgb(230, 57, 70);
    pub const LEGENDARY: Color = Color::Rgb(114, 9, 183);

    // UI elements
    pub const GOLD: Color = Color::Rgb(255, 183, 3);
    pub const DIM_TEXT: Color = Color::Rgb(100, 100, 120);
    pub const BRIGHT_TEXT: Color = Color::Rgb(255, 255, 255);
    pub const MUTED_TEXT: Color = Color::Rgb(160, 160, 180);

    // Blind colors
    pub const SMALL_BLIND: Color = Color::Rgb(76, 201, 240);
    pub const BIG_BLIND: Color = Color::Rgb(255, 183, 3);
    pub const BOSS_BLIND: Color = Color::Rgb(230, 57, 70);
}
