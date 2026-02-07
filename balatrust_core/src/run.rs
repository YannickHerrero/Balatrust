use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::blind::{self, BlindType, BossBlind};
use crate::card::PlayingCard;
use crate::consumable::{Consumable, ConsumableType, TarotCard};
use crate::deck::Deck;
use crate::joker::{Joker, JokerType};
use crate::scoring::HandLevels;
use crate::shop::{Shop, ShopItem};

/// The phase within an ante
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntePhase {
    /// Choosing which blind to play/skip
    BlindSelect,
    /// Playing the current blind
    Playing,
    /// Viewing the shop
    Shop,
}

/// Complete run state
#[derive(Debug, Clone)]
pub struct RunState {
    pub ante: u8,
    pub blind_type: BlindType,
    pub ante_phase: AntePhase,

    pub money: u32,
    pub hands_remaining: u8,
    pub discards_remaining: u8,
    pub hand_size: u8,
    pub max_jokers: u8,
    pub max_consumables: u8,

    pub deck: Deck,
    pub hand: Vec<PlayingCard>,
    pub selected_indices: Vec<usize>,

    // Jokers and consumables
    pub jokers: Vec<Joker>,
    pub consumables: Vec<Consumable>,

    pub hand_levels: HandLevels,
    pub round_score: u64,
    pub score_target: u64,

    pub boss_blind: BossBlind,
    pub rng: StdRng,

    /// Blinds beaten this ante (to track progression)
    pub blinds_beaten: u8,

    /// Shop state
    pub shop: Option<Shop>,
}

impl RunState {
    pub fn new() -> Self {
        Self::with_seed(rand::thread_rng().gen())
    }

    pub fn with_seed(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut deck = Deck::standard();
        deck.shuffle(&mut rng);

        let boss = Self::random_boss(&mut rng);
        let blind_type = BlindType::Small;
        let ante = 1;
        let score_target = blind::score_target(ante, &blind_type);

        Self {
            ante,
            blind_type,
            ante_phase: AntePhase::BlindSelect,
            money: 4,
            hands_remaining: 4,
            discards_remaining: 3,
            hand_size: 8,
            max_jokers: 5,
            max_consumables: 2,
            deck,
            hand: Vec::new(),
            selected_indices: Vec::new(),
            jokers: Vec::new(),
            consumables: Vec::new(),
            hand_levels: HandLevels::new(),
            round_score: 0,
            score_target,
            boss_blind: boss,
            rng,
            blinds_beaten: 0,
            shop: None,
        }
    }

    fn random_boss(rng: &mut StdRng) -> BossBlind {
        let idx = rng.gen_range(0..BossBlind::ALL.len());
        BossBlind::ALL[idx]
    }

    /// Start playing a blind: reset round state and draw hand
    pub fn start_blind(&mut self) {
        self.ante_phase = AntePhase::Playing;
        self.round_score = 0;
        self.hands_remaining = 4;
        self.discards_remaining = 3;
        self.selected_indices.clear();
        self.score_target = blind::score_target(self.ante, &self.blind_type);

        // Apply boss blind effects at start
        if let BlindType::Boss(boss) = &self.blind_type {
            match boss {
                BossBlind::TheNeedle => self.hands_remaining = 1,
                _ => {}
            }
        }

        // Reset deck and draw hand
        self.deck.reset_and_shuffle(&mut self.rng);
        self.hand = self.deck.draw(self.hand_size as usize);

        // Apply suit debuffs from boss blinds
        self.apply_boss_debuffs();
    }

    /// Apply boss blind suit debuffs to hand cards
    fn apply_boss_debuffs(&mut self) {
        if let BlindType::Boss(boss) = &self.blind_type {
            let debuff_suit = match boss {
                BossBlind::TheClub => Some(crate::card::Suit::Clubs),
                BossBlind::TheGoad => Some(crate::card::Suit::Spades),
                BossBlind::TheWindow => Some(crate::card::Suit::Diamonds),
                BossBlind::TheHead => Some(crate::card::Suit::Hearts),
                _ => None,
            };
            if let Some(suit) = debuff_suit {
                for card in &mut self.hand {
                    if card.suit == suit {
                        card.debuffed = true;
                    }
                }
            }
        }
    }

    /// Skip the current blind
    pub fn skip_blind(&mut self) {
        self.advance_blind();
    }

    /// Add score from a hand
    pub fn add_score(&mut self, score: u64) {
        self.round_score += score;
    }

    /// Use a hand (after playing)
    pub fn use_hand(&mut self) {
        if self.hands_remaining > 0 {
            self.hands_remaining -= 1;
        }
    }

    /// Use a discard
    pub fn use_discard(&mut self) -> bool {
        if self.discards_remaining > 0 {
            self.discards_remaining -= 1;
            true
        } else {
            false
        }
    }

    /// Check if the blind is beaten
    pub fn blind_beaten(&self) -> bool {
        self.round_score >= self.score_target
    }

    /// Check if the round is lost (no hands left and target not met)
    pub fn round_lost(&self) -> bool {
        self.hands_remaining == 0 && !self.blind_beaten()
    }

    /// Check if the entire run is won (beat ante 8)
    pub fn run_won(&self) -> bool {
        self.ante > 8
    }

    /// Calculate money earned after beating a blind
    pub fn calculate_reward(&self) -> u32 {
        let mut reward = self.blind_type.reward();
        // +$1 per remaining hand
        reward += self.hands_remaining as u32;
        // Interest: $1 per $5 held, capped at $5
        let interest = (self.money / 5).min(5);
        reward += interest;
        // Golden Joker: +$4 each
        let golden_bonus: u32 = self
            .jokers
            .iter()
            .filter(|j| j.joker_type == JokerType::GoldenJoker)
            .count() as u32
            * 4;
        reward += golden_bonus;
        reward
    }

    /// Beat the current blind and collect rewards, then go to shop
    pub fn beat_blind(&mut self) {
        let reward = self.calculate_reward();
        self.money += reward;
        self.blinds_beaten += 1;

        // Egg joker: +$3 sell value per round
        for joker in &mut self.jokers {
            if joker.joker_type == JokerType::Egg {
                joker.bonus_sell += 3;
            }
        }

        self.ante_phase = AntePhase::Shop;

        // Return hand cards to deck and clear debuffs
        for card in &mut self.hand {
            card.debuffed = false;
        }
        let hand_cards: Vec<PlayingCard> = self.hand.drain(..).collect();
        self.deck.discard_cards(&hand_cards);
        self.selected_indices.clear();

        // Generate shop
        self.shop = Some(Shop::generate(&mut self.rng, self.ante));
    }

    /// Leave the shop and advance to next blind
    pub fn leave_shop(&mut self) {
        self.shop = None;
        self.advance_blind();
    }

    /// Advance to the next blind in sequence
    fn advance_blind(&mut self) {
        match self.blind_type {
            BlindType::Small => {
                self.blind_type = BlindType::Big;
            }
            BlindType::Big => {
                self.blind_type = BlindType::Boss(self.boss_blind);
            }
            BlindType::Boss(_) => {
                // Completed the ante, advance
                self.ante += 1;
                self.blinds_beaten = 0;
                self.boss_blind = Self::random_boss(&mut self.rng);
                self.blind_type = BlindType::Small;
            }
        }
        self.ante_phase = AntePhase::BlindSelect;
        self.score_target = blind::score_target(self.ante, &self.blind_type);
    }

    /// Remove selected cards from hand and draw replacements
    pub fn discard_selected(&mut self) -> Vec<PlayingCard> {
        let mut discarded = Vec::new();
        let mut indices: Vec<usize> = self.selected_indices.clone();
        indices.sort_unstable_by(|a, b| b.cmp(a));

        for &idx in &indices {
            if idx < self.hand.len() {
                discarded.push(self.hand.remove(idx));
            }
        }

        self.deck.discard_cards(&discarded);
        self.selected_indices.clear();

        // Draw replacements
        let need = (self.hand_size as usize).saturating_sub(self.hand.len());
        let mut drawn = self.deck.draw(need);
        self.hand.append(&mut drawn);

        // Apply debuffs to new cards
        self.apply_boss_debuffs();

        discarded
    }

    /// Play selected cards: remove them from hand, return the played cards
    pub fn play_selected(&mut self) -> Vec<PlayingCard> {
        let mut played = Vec::new();
        let mut indices: Vec<usize> = self.selected_indices.clone();
        indices.sort_unstable_by(|a, b| b.cmp(a));

        for &idx in &indices {
            if idx < self.hand.len() {
                played.push(self.hand.remove(idx));
            }
        }

        // Reverse to get original order
        played.reverse();
        self.selected_indices.clear();

        played
    }

    /// Draw cards to fill hand back to hand_size
    pub fn draw_to_hand_size(&mut self) {
        let need = (self.hand_size as usize).saturating_sub(self.hand.len());
        if need > 0 {
            let mut drawn = self.deck.draw(need);
            self.hand.append(&mut drawn);
            self.apply_boss_debuffs();
        }
    }

    /// Toggle selection of a card at index
    pub fn toggle_select(&mut self, idx: usize) {
        if idx >= self.hand.len() {
            return;
        }
        if let Some(pos) = self.selected_indices.iter().position(|&i| i == idx) {
            self.selected_indices.remove(pos);
        } else if self.selected_indices.len() < 5 {
            self.selected_indices.push(idx);
        }
    }

    /// Check if a hand index is selected
    pub fn is_selected(&self, idx: usize) -> bool {
        self.selected_indices.contains(&idx)
    }

    /// Get the currently selected cards (in selection order)
    pub fn selected_cards(&self) -> Vec<PlayingCard> {
        self.selected_indices
            .iter()
            .filter_map(|&i| self.hand.get(i).copied())
            .collect()
    }

    /// Can the player play a hand right now?
    pub fn can_play(&self) -> bool {
        if self.hands_remaining == 0 || self.selected_indices.is_empty() {
            return false;
        }
        if self.selected_indices.len() > 5 {
            return false;
        }
        // The Psychic: must play exactly 5 cards
        if let BlindType::Boss(BossBlind::ThePsychic) = &self.blind_type {
            if self.selected_indices.len() != 5 {
                return false;
            }
        }
        true
    }

    /// Apply The Hook effect: discard 2 random cards from hand
    pub fn apply_hook_effect(&mut self) {
        if let BlindType::Boss(BossBlind::TheHook) = &self.blind_type {
            use rand::seq::SliceRandom;
            if self.hand.len() > 2 {
                let mut indices: Vec<usize> = (0..self.hand.len()).collect();
                indices.shuffle(&mut self.rng);
                let to_remove: Vec<usize> = indices.into_iter().take(2).collect();
                let mut sorted = to_remove;
                sorted.sort_unstable_by(|a, b| b.cmp(a));
                for idx in sorted {
                    let card = self.hand.remove(idx);
                    self.deck.discard_cards(&[card]);
                }
            }
        }
    }

    /// Can the player discard right now?
    pub fn can_discard(&self) -> bool {
        self.discards_remaining > 0 && !self.selected_indices.is_empty()
    }

    /// Buy a shop item
    pub fn buy_shop_item(&mut self, index: usize) -> bool {
        let price = if let Some(shop) = &self.shop {
            if let Some(item) = shop.items.get(index) {
                item.price()
            } else {
                return false;
            }
        } else {
            return false;
        };

        if self.money < price {
            return false;
        }

        // Check capacity
        if let Some(shop) = &self.shop {
            match &shop.items.get(index) {
                Some(ShopItem::JokerItem(_)) => {
                    if self.jokers.len() >= self.max_jokers as usize {
                        return false;
                    }
                }
                Some(ShopItem::ConsumableItem(_)) => {
                    if self.consumables.len() >= self.max_consumables as usize {
                        return false;
                    }
                }
                None => return false,
            }
        }

        if let Some(shop) = &mut self.shop {
            if let Some(item) = shop.buy(index) {
                self.money -= price;
                match item {
                    ShopItem::JokerItem(joker) => {
                        self.jokers.push(joker);
                    }
                    ShopItem::ConsumableItem(consumable) => {
                        self.consumables.push(consumable);
                    }
                }
                return true;
            }
        }
        false
    }

    /// Sell a joker
    pub fn sell_joker(&mut self, index: usize) -> bool {
        if index >= self.jokers.len() {
            return false;
        }
        let joker = self.jokers.remove(index);
        self.money += joker.total_sell_value();
        true
    }

    /// Reroll the shop
    pub fn reroll_shop(&mut self) -> bool {
        let cost = if let Some(shop) = &self.shop {
            shop.reroll_cost
        } else {
            return false;
        };

        if self.money < cost {
            return false;
        }

        self.money -= cost;
        let ante = self.ante;
        if let Some(shop) = &mut self.shop {
            let old_cost = shop.reroll_cost;
            *shop = Shop::generate(&mut self.rng, ante);
            shop.reroll_cost = old_cost + 1;
        }
        true
    }

    /// Use a consumable (planet card - level up)
    pub fn use_planet(&mut self, consumable_index: usize) -> bool {
        if consumable_index >= self.consumables.len() {
            return false;
        }
        if let ConsumableType::Planet(planet) = self.consumables[consumable_index].consumable_type {
            self.hand_levels.level_up(planet.hand_type());
            self.consumables.remove(consumable_index);
            true
        } else {
            false
        }
    }

    /// Use a tarot card (apply enhancement to selected cards)
    pub fn use_tarot(&mut self, consumable_index: usize) -> bool {
        if consumable_index >= self.consumables.len() {
            return false;
        }
        if let ConsumableType::Tarot(tarot) = self.consumables[consumable_index].consumable_type {
            let (min, max) = tarot.cards_needed();
            let selected_count = self.selected_indices.len();

            // Check card count requirement
            if min > 0 && (selected_count < min || selected_count > max) {
                return false;
            }

            match tarot {
                TarotCard::TheHierophant => {
                    // Enhance to Bonus
                    for &idx in &self.selected_indices.clone() {
                        if idx < self.hand.len() {
                            self.hand[idx].enhancement = Some(crate::card::Enhancement::Bonus);
                        }
                    }
                }
                TarotCard::TheEmpress => {
                    // Enhance to Mult
                    for &idx in &self.selected_indices.clone() {
                        if idx < self.hand.len() {
                            self.hand[idx].enhancement = Some(crate::card::Enhancement::Mult);
                        }
                    }
                }
                TarotCard::TheMagician => {
                    // Enhance to Lucky
                    for &idx in &self.selected_indices.clone() {
                        if idx < self.hand.len() {
                            self.hand[idx].enhancement = Some(crate::card::Enhancement::Lucky);
                        }
                    }
                }
                TarotCard::TheLover => {
                    // Enhance to Wild
                    if let Some(&idx) = self.selected_indices.first() {
                        if idx < self.hand.len() {
                            self.hand[idx].enhancement = Some(crate::card::Enhancement::Wild);
                        }
                    }
                }
                TarotCard::TheChariot => {
                    // Enhance to Steel
                    if let Some(&idx) = self.selected_indices.first() {
                        if idx < self.hand.len() {
                            self.hand[idx].enhancement = Some(crate::card::Enhancement::Steel);
                        }
                    }
                }
                TarotCard::Strength => {
                    // Increase rank by 1
                    for &idx in &self.selected_indices.clone() {
                        if idx < self.hand.len() {
                            let new_rank = match self.hand[idx].rank {
                                crate::card::Rank::Two => crate::card::Rank::Three,
                                crate::card::Rank::Three => crate::card::Rank::Four,
                                crate::card::Rank::Four => crate::card::Rank::Five,
                                crate::card::Rank::Five => crate::card::Rank::Six,
                                crate::card::Rank::Six => crate::card::Rank::Seven,
                                crate::card::Rank::Seven => crate::card::Rank::Eight,
                                crate::card::Rank::Eight => crate::card::Rank::Nine,
                                crate::card::Rank::Nine => crate::card::Rank::Ten,
                                crate::card::Rank::Ten => crate::card::Rank::Jack,
                                crate::card::Rank::Jack => crate::card::Rank::Queen,
                                crate::card::Rank::Queen => crate::card::Rank::King,
                                crate::card::Rank::King => crate::card::Rank::Ace,
                                crate::card::Rank::Ace => crate::card::Rank::Two,
                            };
                            self.hand[idx].rank = new_rank;
                        }
                    }
                }
                TarotCard::TheHermit => {
                    // Double money, max $20
                    let gain = self.money.min(20);
                    self.money += gain;
                }
                TarotCard::Temperance => {
                    // Gain $ equal to joker sell values, max $50
                    let total_sell: u32 = self.jokers.iter().map(|j| j.total_sell_value()).sum();
                    self.money += total_sell.min(50);
                }
                _ => {
                    // Other tarots not fully implemented yet
                }
            }

            self.consumables.remove(consumable_index);
            true
        } else {
            false
        }
    }
}
