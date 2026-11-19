use serde::{Deserialize, Serialize};

use crate::mob::DamageType;
use crate::skill::Skill;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EquipLocation {
    Head,
    Chest,
    Gloves,
    Legs,
    Feet,
    Neck,
    Finger,
    Weapon,
    Weapon2h,
    Bag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Legendary,
    Never, // shop or quest only; never drops
}

impl Rarity {
    /// Relative chance of being picked. Higher means more likely.
    pub fn weight(self) -> u32 {
        match self {
            Rarity::Common => 100,
            Rarity::Uncommon => 40,
            Rarity::Rare => 10,
            Rarity::Legendary => 2,
            Rarity::Never => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
    Damage { amount: u32, damage_type: DamageType }, // Cause damage to a mob
    Heal { amount: u32 }, // Increase hits to self
    Defend {
        amount: i32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        damage_type: Option<DamageType> }, // Damage reduction for a specific type, None is all types
    Skill { amount: i32, skill: Skill }, // Add to skill
    Capacity { slots: u32 },
    Custom { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub codex: String,
    pub level: i32,
    pub value: u32,
    #[serde(default)]
    pub rarity: Rarity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equip_to: Option<EquipLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_rate: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub stackable: bool,
}