#![allow(dead_code)]

use ratatui::style::Color;
use tachyonfx::fx;
use tachyonfx::{Effect, EffectManager, Interpolation, IntoEffect, Motion};

/// Our keyed effect manager using tachyonfx's built-in EffectManager
pub type FxManager = EffectManager<&'static str>;

// ─── Effect Factories ────────────────────────────────────────────────

const DARK: Color = Color::Rgb(13, 17, 23);
const FELT: Color = Color::Rgb(22, 22, 52);

/// Screen transition: content sweeps in from the left
pub fn screen_transition() -> Effect {
    fx::sweep_in(
        Motion::LeftToRight,
        8,
        2,
        DARK,
        (400, Interpolation::CubicOut),
    )
}

/// Coalesce effect: content materializes from empty space
pub fn coalesce_in() -> Effect {
    fx::coalesce((450, Interpolation::CubicOut))
}

/// Score reveal: sweep from left with a quick timing
pub fn score_sweep() -> Effect {
    fx::sweep_in(
        Motion::LeftToRight,
        4,
        1,
        DARK,
        (300, Interpolation::QuadOut),
    )
}

/// Celebration HSL shift for "Blind Defeated" popup
pub fn celebration_shimmer() -> Effect {
    let shift = fx::hsl_shift_fg([30.0, 0.0, 0.15], (800, Interpolation::SineInOut));
    fx::ping_pong(shift)
}

/// Subtle gold shimmer for the title on main menu
pub fn title_shimmer() -> Effect {
    let shift = fx::hsl_shift_fg([15.0, 0.1, 0.1], (1200, Interpolation::SineInOut));
    fx::repeating(fx::ping_pong(shift))
}

/// Glitch effect for boss blinds
pub fn boss_glitch() -> Effect {
    fx::Glitch::builder()
        .cell_glitch_ratio(0.015)
        .action_start_delay_ms(200..1500)
        .action_ms(50..200)
        .build()
        .into_effect()
}

/// Dissolve out (for leaving a screen)
pub fn dissolve_out() -> Effect {
    fx::dissolve((300, Interpolation::QuadIn))
}

/// Slide in from below for cards being dealt
pub fn card_deal_slide() -> Effect {
    fx::slide_in(Motion::DownToUp, 3, 1, FELT, (350, Interpolation::CubicOut))
}

/// Fade foreground to gold (for score number highlight)
pub fn score_highlight() -> Effect {
    let gold = Color::Rgb(255, 214, 10);
    let shift = fx::fade_to_fg(gold, (200, Interpolation::QuadOut));
    let shift_back = fx::fade_from_fg(gold, (600, Interpolation::QuadIn));
    fx::sequence(&[shift, shift_back])
}
