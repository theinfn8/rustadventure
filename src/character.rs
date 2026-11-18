use serde::{Deserialize, Serialize};

use crate::skill::Skills;
use crate::inventory::{Inventory, InventoryItem};
use crate::item::{Effect, EquipLocation};
use crate::mob::DamageType;

pub const BASE_INVENTORY_SLOTS: u32 = 4;

// Equip source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Bag,
    Bank,
}

// For equip failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipError {
    NoSuchItem,
    NotEquippable,
    WrongSlot,
}

#[derive(Debug, Default)]
pub struct GearChange {
    pub removed: Vec<InventoryItem>,
    pub overflow: Vec<InventoryItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Head,
    Chest,
    Gloves,
    Legs,
    Feet,
    Neck,
    Finger1,
    Finger2,
    MainHand,
    OffHand,
    Bag,
}

impl Slot {
    pub fn for_location(loc: EquipLocation) -> &'static [Slot] {
        match loc {
            EquipLocation::Head => &[Slot::Head],
            EquipLocation::Chest => &[Slot::Chest],
            EquipLocation::Gloves => &[Slot::Gloves],
            EquipLocation::Legs => &[Slot::Legs],
            EquipLocation::Feet => &[Slot::Feet],
            EquipLocation::Neck => &[Slot::Neck],
            EquipLocation::Finger => &[Slot::Finger1, Slot::Finger2],
            EquipLocation::Weapon => &[Slot::MainHand, Slot::OffHand],
            EquipLocation::Weapon2h => &[Slot::MainHand],
            EquipLocation::Bag => &[Slot::Bag],
            
        }
    }

    pub const ALL: [Slot; 11] = [
        Slot::Head,
        Slot::Neck,
        Slot::Chest,
        Slot::Gloves,
        Slot::Finger1,
        Slot::Finger2,
        Slot::Legs,
        Slot::Feet,
        Slot::MainHand,
        Slot::OffHand,
        Slot::Bag,
    ];
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub head: Option<InventoryItem>,
    pub chest: Option<InventoryItem>,
    pub gloves: Option<InventoryItem>,
    pub legs: Option<InventoryItem>,
    pub feet: Option<InventoryItem>,
    pub neck: Option<InventoryItem>,
    pub finger1: Option<InventoryItem>,
    pub finger2: Option<InventoryItem>,
    pub main_hand: Option<InventoryItem>,
    pub off_hand: Option<InventoryItem>,
    pub bag: Option<InventoryItem>,
}

impl Equipment {
    pub fn get_mut(&mut self, slot: Slot) -> &mut Option<InventoryItem> {
        match slot {
            Slot::Head => &mut self.head,
            Slot::Chest => &mut self.chest,
            Slot::Gloves => &mut self.gloves,
            Slot::Legs => &mut self.legs,
            Slot::Feet => &mut self.feet,
            Slot::Neck => &mut self.neck,
            Slot::Finger1 => &mut self.finger1,
            Slot::Finger2 => &mut self.finger2,
            Slot::MainHand => &mut self.main_hand,
            Slot::OffHand => &mut self.off_hand,
            Slot::Bag => &mut self.bag,
        }
    }

    pub fn get(&self, slot: Slot) -> &Option<InventoryItem> {
        match slot {
            Slot::Head => &self.head,
            Slot::Chest => &self.chest,
            Slot::Gloves => &self.gloves,
            Slot::Legs => &self.legs,
            Slot::Feet => &self.feet,
            Slot::Neck => &self.neck,
            Slot::Finger1 => &self.finger1,
            Slot::Finger2 => &self.finger2,
            Slot::MainHand => &self.main_hand,
            Slot::OffHand => &self.off_hand,
            Slot::Bag => &self.bag,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &InventoryItem> {
        Slot::ALL.into_iter().filter_map(|slot| self.get(slot).as_ref())
    }

    pub fn effects(&self) -> impl Iterator<Item = &'static Effect> + '_ {
        self.iter().flat_map(|entry| entry.template().effects.iter())
    }

    // Check for two-handed weapon blocking off hand
    pub fn off_hand_blocked(&self) -> bool {
        self.main_hand
            .as_ref()
            .is_some_and(|e| e.template().equip_to == Some(EquipLocation::Weapon2h))
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub level: u32,
    pub xp: u32,
    pub max_hits: u32,
    pub current_hits: u32,
    pub skills: Skills,
    pub gold: u32,
    pub bank_gold: u32,
    pub gear: Equipment,
    pub inventory: Inventory,
    pub bank: Inventory,
}

impl Character {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            level: 1,
            xp: 0,
            max_hits: 20,
            current_hits: 20,
            skills: Skills { combat: 2, defense: 2, arcana: 2, thievery: 2 },
            gold: 25,
            bank_gold: 0,
            gear: Equipment::default(),
            inventory: Inventory::with_slots(BASE_INVENTORY_SLOTS),
            bank: Inventory::unlimited(),
        }
    }

    // Move gold from character to bank, capping to source available amount
    pub fn deposit_gold(&mut self, amount: u32) -> u32 {
        let moved = amount.min(self.gold);
        self.gold -= moved;
        self.bank_gold += moved;
        moved
    }

    // Move gold from bank to character, capping at source available amount
    pub fn withdraw_gold(&mut self, amount: u32) -> u32 {
        let moved = amount.min(self.bank_gold);
        self.bank_gold -= moved;
        self.gold += moved;
        moved
    }

    // Equip item
    pub fn equip(&mut self, from: Source, index: usize, slot: Option<Slot>) -> Result<GearChange, EquipError> {
        // Look first, so nothing moves if the request is invalid.
        let entry = match from {
            Source::Bag => self.inventory.get(index),
            Source::Bank => self.bank.get(index),
        }
        .ok_or(EquipError::NoSuchItem)?;
        let location = entry.template().equip_to.ok_or(EquipError::NotEquippable)?;
        let target = self.choose_slot(location, slot)?;
 
        // Take one copy out of the source.
        let item = match from {
            Source::Bag => self.inventory.remove(index, 1),
            Source::Bank => self.bank.remove(index, 1),
        }
        .ok_or(EquipError::NoSuchItem)?;
 
        // Clear whatever the new item displaces.
        let mut change = GearChange::default();
        let clear: &[Slot] = if location == EquipLocation::Weapon2h {
            &[Slot::MainHand, Slot::OffHand] // two-handed: empty both hands
        } else if target == Slot::OffHand && self.gear.off_hand_blocked() {
            &[Slot::MainHand] // off hand blocked by a two-handed weapon: remove it
        } else {
            std::slice::from_ref(&target)
        };
        for &s in clear {
            if let Some(old) = self.gear.get_mut(s).take() {
                change.removed.push(old);
            }
        }
 
        *self.gear.get_mut(target) = Some(item);
 
        self.store_in_bank(&change.removed);
        change.overflow = self.resize_bag();
        Ok(change)
    }

    // Unequip an item and dump it in the bank
    pub fn unequip(&mut self, slot: Slot) -> Option<GearChange> {
        let old = self.gear.get_mut(slot).take()?;
        let mut change = GearChange { removed: vec![old], overflow: Vec::new() };
        self.store_in_bank(&change.removed);
        change.overflow = self.resize_bag();
        Some(change)
    }

    // Identify slot to use
    fn choose_slot(&self, location: EquipLocation, requested: Option<Slot>) -> Result<Slot, EquipError> {
        let candidates = Slot::for_location(location);
        if let Some(slot) = requested {
            return if candidates.contains(&slot) { Ok(slot) } else { Err(EquipError::WrongSlot) };
        }
        let empty = candidates.iter().copied().find(|&s| {
            self.gear.get(s).is_none() && !(s == Slot::OffHand && self.gear.off_hand_blocked())
        });
        Ok(empty.unwrap_or(candidates[0]))
    }

    // ... Store item in the bank...
    fn store_in_bank(&mut self, items: &[InventoryItem]) {
        for item in items {
            // The bank is unlimited, so this can't fail.
            self.bank.add(item.clone()).expect("bank is full");
        }
    }

    fn resize_bag(&mut self) -> Vec<InventoryItem> {
        let overflow = self.inventory.set_max_slots(Some(self.inventory_slots()));
        self.store_in_bank(&overflow);
        overflow
    }

    pub fn is_dead(&self) -> bool {
        self.current_hits == 0
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.current_hits = self.current_hits.saturating_sub(amount);
    }

    pub fn heal(&mut self, amount: u32) {
        self.current_hits = (self.current_hits + amount).min(self.max_hits);
    }

    pub fn total_skills(&self) -> Skills {
        let mut total = self.skills;
        for effect in self.gear.effects() {
            if let Effect::Skill { amount, skill } = effect {
                *total.get_mut(*skill) += amount;
            }
        }
        total
    }

    pub fn defense_against(&self, damage_type: DamageType) -> u32 {
        let total: i32 = self
            .gear
            .effects()
            .filter_map(|effect| match effect {
                Effect::Defend { amount, damage_type: dt } if dt.is_none_or(|t| t == damage_type) => {
                    Some(*amount)
                }
                _ => None,
            })
            .sum();
        total.max(0) as u32
    }

    pub fn damage_after_defense(&self, amount: u32, damage_type: DamageType) -> u32 {
        amount.saturating_sub(self.defense_against(damage_type))
    }

    pub fn inventory_slots(&self) -> u32 {
        let from_bag: u32 = self
            .gear
            .bag
            .iter()
            .flat_map(|entry| entry.template().effects.iter())
            .filter_map(|effect| match effect {
                Effect::Capacity { slots } => Some(*slots),
                _ => None,
            })
            .sum();
        BASE_INVENTORY_SLOTS + from_bag
    }

}