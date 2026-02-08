# Balatrust

A terminal-native, open-source Balatro-inspired roguelike deckbuilder written in Rust.

Balatrust recreates the core run loop of building score through poker hands, jokers, consumables, and shop decisions, with a polished TUI experience powered by Ratatui + Crossterm.

## Features

- Full run flow: Main Menu -> Blind Select -> Round -> Shop -> Game Over
- Ante progression with Small, Big, and Boss blinds
- Boss mechanics including The Hook, The Wall, The Psychic, The Needle, and suit-debuff bosses
- Poker-hand detection and step-by-step animated scoring pipeline
- 20 implemented jokers with rarity, pricing, effects, and sell value
- Planet and Tarot consumables (including hand-level upgrades and several card/money effects)
- Shop flow with buying, rerolling, joker selling, and capacity limits
- Keyboard + mouse support for core gameplay and shop interactions
- Visual polish: animated background, transitions, score popups, and effect pulses

## Installation

### Requirements

- Rust (stable toolchain)
- Cargo

### Install directly from GitHub

```bash
cargo install --git https://github.com/YannickHerrero/Balatrust.git --bin balatrust
```

Then run:

```bash
balatrust
```

### Run from source (development)

```bash
git clone https://github.com/YannickHerrero/Balatrust.git
cd Balatrust
```

Build and run:

```bash
cargo run -p balatrust
```

### Run tests

```bash
cargo test
```

## Controls

### Global

- `Ctrl+C`: Quit from anywhere

### Main Menu

- `Up/Down` or `j/k`: Navigate
- `Enter`: Select
- `q`: Quit

### Blind Select

- `Left/Right` or `h/l`: Move between blinds
- `Enter`: Start blind
- `s`: Skip current blind (Small/Big)

### Round (Playing)

- `Left/Right` or `h/l`: Move hand cursor
- `Space` or `Up/k`: Toggle selected card
- `Enter` or `p`: Play selected cards
- `d`: Discard selected cards
- `s`: Sort hand by rank
- `t`: Sort hand by suit
- `a`: Select up to 5 cards
- `c`: Clear selection
- Mouse: select cards, press action buttons, inspect jokers

### Round (Scoring Animation)

- `Space`, `Enter`, or `p`: Skip scoring animation

### Shop

- `Tab`: Switch focus (items/jokers)
- `Left/Right` or `h/l`: Move cursor
- `Enter` or `Space`: Inspect item/joker (and confirm buy in item popup)
- `r`: Reroll shop
- `s`: Sell selected joker
- `n`: Leave shop / next round
- Mouse: inspect cards, buy, reroll, next round

## Gameplay Loop

1. Start a run and choose blinds.
2. Build score by playing poker hands before hands run out.
3. Beat the blind target, cash out, and collect rewards.
4. Visit the shop to buy jokers/consumables, reroll, and optimize your setup.
5. Push through antes and boss blinds until victory (or game over).

## Project Structure

- `balatrust/`: executable app, screen flow, input handling, effects orchestration
- `balatrust_core/`: game rules, run state, cards, blinds, scoring, jokers, shop, consumables
- `balatrust_widgets/`: reusable Ratatui widgets and visual theme components

The repository is organized as a Cargo workspace with these three crates.

## Roadmap

- Expand feature parity with more Balatro mechanics
- Complete and refine currently partial Tarot behavior
- Add balancing passes and broader content variety
- Improve testing coverage beyond core logic modules

## Contributing

Contributions are welcome.

- Open an issue to discuss bugs, balance ideas, or features
- Submit focused pull requests with clear descriptions
- Run tests before opening a PR (`cargo test`)

Project creator: Yannick Herrero ([`@YannickHerrero`](https://github.com/YannickHerrero))

## License

This project is licensed under the MIT License. See `LICENSE` for details.

## Disclaimer

Balatrust is an unofficial fan project inspired by Balatro and is not affiliated with or endorsed by the original creators.
