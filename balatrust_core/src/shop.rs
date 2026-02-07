use rand::Rng;

use crate::consumable::{Consumable, PlanetCard, TarotCard};
use crate::joker::{Joker, JokerType};

/// An item available in the shop
#[derive(Debug, Clone)]
pub enum ShopItem {
    JokerItem(Joker),
    ConsumableItem(Consumable),
}

impl ShopItem {
    pub fn name(&self) -> String {
        match self {
            ShopItem::JokerItem(j) => j.joker_type.name().to_string(),
            ShopItem::ConsumableItem(c) => c.consumable_type.name().to_string(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            ShopItem::JokerItem(j) => j.joker_type.description().to_string(),
            ShopItem::ConsumableItem(c) => c.consumable_type.description(),
        }
    }

    pub fn price(&self) -> u32 {
        match self {
            ShopItem::JokerItem(j) => j.joker_type.price(),
            ShopItem::ConsumableItem(c) => c.consumable_type.price(),
        }
    }
}

/// The shop state
#[derive(Debug, Clone)]
pub struct Shop {
    pub items: Vec<ShopItem>,
    pub reroll_cost: u32,
}

impl Shop {
    pub fn generate<R: Rng>(rng: &mut R, _ante: u8) -> Self {
        let mut items = Vec::new();

        // Generate 2 items: ~70% joker, ~15% planet, ~15% tarot
        for _ in 0..2 {
            let roll: f32 = rng.gen();
            if roll < 0.70 {
                // Random joker
                let idx = rng.gen_range(0..JokerType::ALL.len());
                items.push(ShopItem::JokerItem(Joker::new(JokerType::ALL[idx])));
            } else if roll < 0.85 {
                // Random planet
                let idx = rng.gen_range(0..PlanetCard::COMMON.len());
                items.push(ShopItem::ConsumableItem(Consumable::planet(
                    PlanetCard::COMMON[idx],
                )));
            } else {
                // Random tarot
                let idx = rng.gen_range(0..TarotCard::ALL.len());
                items.push(ShopItem::ConsumableItem(Consumable::tarot(
                    TarotCard::ALL[idx],
                )));
            }
        }

        Self {
            items,
            reroll_cost: 5,
        }
    }

    pub fn reroll<R: Rng>(&mut self, rng: &mut R, ante: u8) {
        *self = Self::generate(rng, ante);
        self.reroll_cost += 1; // Stays incremented
    }

    pub fn buy(&mut self, index: usize) -> Option<ShopItem> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }
}
