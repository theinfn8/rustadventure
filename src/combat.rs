use serde::{Deserialize, Serialize};

use crate::mob::DamageType;

/// One component of an attack: this much damage of this type.
/// A flaming dagger's swing is two of these: some Physical and some Fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Damage {
    pub amount: u32,
    pub damage_type: DamageType,
}

impl Damage {
    pub fn new(amount: u32, damage_type: DamageType) -> Self {
        Self { amount, damage_type }
    }
}

/// How a mob reacts to one damage type. Types a mob doesn't list take normal damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Affinity {
    /// Takes no damage.
    Immune,
    /// Takes half, rounded up, so a resisted hit always does at least 1.
    Resistant,
    /// Takes 1.5 times as much, rounded up, so a weakness always adds at least 1.
    Weak,
}

impl Affinity {
    /// The damage that actually lands after this affinity.
    pub fn apply(self, amount: u32) -> u32 {
        match self {
            Affinity::Immune => 0,
            Affinity::Resistant => amount.div_ceil(2),
            Affinity::Weak => (amount * 3).div_ceil(2),
        }
    }
}

/// The result of one damage component landing on an enemy.
#[derive(Debug, Clone, Copy)]
pub struct Hit {
    /// What was attempted.
    pub damage: Damage,
    /// What actually landed.
    pub dealt: u32,
    /// The affinity that changed it, if any.
    pub affinity: Option<Affinity>,
}