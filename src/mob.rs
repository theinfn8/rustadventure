use rand::seq::IndexedRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    Physical,
    Air,
    Earth,
    Water,
    Fire,
    Light,
    Dark,
}

impl fmt::Display for DamageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DamageType::Physical => "Physical",
            DamageType::Air => "Air",
            DamageType::Earth => "Earth",
            DamageType::Water => "Water",
            DamageType::Fire => "Fire",
            DamageType::Light => "Light",
            DamageType::Dark => "Dark",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attack {
    pub name: String,
    pub prep_text: String,
    pub damage: u32,
    pub damage_type: DamageType,
    pub rate: u32,
}

impl Attack {
    pub fn prep_tick(&self) -> u32 {
        self.rate / 2
    }

    pub fn attack_text(&self, mob_name: &str) -> String {
        format!("{mob_name} attacks with {} doing {} damage!", self.name, self.damage)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mob {
    pub id: String,
    pub name: String,
    pub codex: String,
    pub level: u32,
    pub max_hits: u32,
    
    pub treasure_code: u32,
    pub attacks: Vec<Attack>,
}

impl Mob {
    pub fn choose_attack<'a>(&'a self, rng: &mut impl Rng) -> Option<&'a Attack> {
        self.attacks.choose(rng)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.attacks.is_empty() {
            return Err(format!("Mob {} has no attacks defined.", self.id));
        }
        if self.max_hits == 0 {
            return Err(format!("Mob {} has max_hits set to 0.", self.id));
        }
        for a in &self.attacks {
            if a.rate == 0 {
                return Err(format!("Attack {} of mob {} has rate set to 0.", a.name, self.id));
            }
        }
        Ok(())
    }
}

impl fmt::Display for Mob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (Lv {}, Max Hits: {})", self.name, self.level, self.max_hits)
    }
}