use serde::{Deserialize, Serialize};

use crate::data;
use crate::item::Item;

/// One entry in a bag or bank: which item, plus what's specific to this copy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item_id: String,
    /// Remaining uses for charged items; None for unlimited or stackable items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charges: Option<u32>,
    /// How many are in this stack. Always 1 for non-stackable items.
    pub quantity: u32,
}

impl InventoryItem {
    /// A fresh copy of an item: full charges, quantity 1.
    /// Returns None if the ID isn't in items.json.
    pub fn new(item_id: &str) -> Option<Self> {
        let template = data::item(item_id)?;
        Some(Self {
            item_id: item_id.to_string(),
            charges: if template.stackable { None } else { template.uses },
            quantity: 1,
        })
    }

    /// The item's template from items.json.
    pub fn template(&self) -> &'static Item {
        data::item(&self.item_id).expect("inventory refers to an unknown item")
    }

    /// Display text, e.g. "Healing Potion x3", "Wand of Sparks (2/5)", or just the name.
    pub fn label(&self) -> String {
        let t = self.template();
        match (self.quantity, self.charges, t.uses) {
            (q, _, _) if q > 1 => format!("{} x{q}", t.name),
            (_, Some(left), Some(max)) if !t.stackable => format!("{} ({left}/{max})", t.name),
            _ => t.name.clone(),
        }
    }
}

/// A bag or bank. Items can only be added through `add`, so the slot limit always holds.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inventory {
    items: Vec<InventoryItem>,
    /// Maximum number of entries (stacks count as one). None = unlimited, e.g. the bank.
    max_slots: Option<u32>,
}

impl Inventory {
    /// A limited inventory, such as the bag.
    pub fn with_slots(max_slots: u32) -> Self {
        Self { items: Vec::new(), max_slots: Some(max_slots) }
    }

    /// An inventory with no limit, such as the bank.
    pub fn unlimited() -> Self {
        Self { items: Vec::new(), max_slots: None }
    }

    /// Number of slots in use.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// True when no new entry can be added. A stackable item may still fit onto an existing stack.
    pub fn is_full(&self) -> bool {
        match self.max_slots {
            Some(max) => self.items.len() >= max as usize,
            None => false,
        }
    }

    pub fn max_slots(&self) -> Option<u32> {
        self.max_slots
    }

    /// Read-only access for display (e.g. building a ratatui List).
    pub fn iter(&self) -> impl Iterator<Item = &InventoryItem> {
        self.items.iter()
    }

    pub fn get(&self, index: usize) -> Option<&InventoryItem> {
        self.items.get(index)
    }

    /// Add an entry. Stackable items join an existing stack with the same ID, which needs no
    /// free slot. If there's no room, the entry is handed back in `Err` so nothing is lost:
    /// the caller can offer a swap or leave it behind.
    pub fn add(&mut self, entry: InventoryItem) -> Result<(), InventoryItem> {
        if entry.template().stackable {
            if let Some(stack) = self.items.iter_mut().find(|i| i.item_id == entry.item_id) {
                stack.quantity += entry.quantity;
                return Ok(());
            }
        }
        if self.is_full() {
            return Err(entry);
        }
        self.items.push(entry);
        Ok(())
    }

    /// Take `quantity` items out of the entry at `index` (the index a UI list selected).
    /// From a larger stack, this splits off part of it; otherwise the whole entry is removed.
    /// Returns None if the index is invalid or quantity is 0.
    pub fn remove(&mut self, index: usize, quantity: u32) -> Option<InventoryItem> {
        let entry = self.items.get_mut(index)?;
        if quantity == 0 {
            return None;
        }
        if quantity < entry.quantity {
            entry.quantity -= quantity;
            let mut split = entry.clone();
            split.quantity = quantity;
            Some(split)
        } else {
            Some(self.items.remove(index))
        }
    }

    /// Change the slot limit, e.g. after equipping a different bag.
    /// Entries that no longer fit are removed, most recently added first, and returned
    /// so the caller can move them to the bank.
    pub fn set_max_slots(&mut self, max_slots: Option<u32>) -> Vec<InventoryItem> {
        self.max_slots = max_slots;
        let mut overflow = Vec::new();
        if let Some(max) = max_slots {
            while self.items.len() > max as usize {
                if let Some(entry) = self.items.pop() {
                    overflow.push(entry);
                }
            }
        }
        overflow
    }
}