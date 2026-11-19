use crate::deck::Deck;
use crate::skill::{Skill, Skills};

/// How a run ended. Decides what happens to what was found along the way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunEnd {
    /// Walked out: from the stairs, the boss door, or between rooms.
    Left,
    /// Ran from a fight.
    Fled,
    /// Hits reached 0.
    Died,
}

/// Everything about one trip into the dungeon. Created at the entrance, dropped when the
/// player leaves, so the next trip always starts fresh: new deck, full pools.
#[derive(Debug, Clone)]
pub struct DungeonRun {
    /// The current floor's rooms.
    pub deck: Deck,
    /// Skill points left to spend this run.
    pub pools: Skills,
    /// The pools' starting values (base skills + gear when the run began).
    pub max_pools: Skills,
    /// Gold found this run. Only added to the character's gold on a safe exit.
    pub gold_found: u32,
}

impl DungeonRun {
    /// Start a run on `deck`'s floor with full pools.
    pub fn new(deck: Deck, max_pools: Skills) -> Self {
        Self { deck, pools: max_pools, max_pools, gold_found: 0 }
    }

    pub fn floor(&self) -> u32 {
        self.deck.floor
    }

    /// Move on to the next floor. Pools are NOT refilled: that's what limits a run.
    pub fn next_floor(&mut self, deck: Deck) {
        self.deck = deck;
    }

    /// Spend `cost` points from a skill pool. Returns false (and spends nothing)
    /// if there aren't enough left.
    pub fn spend(&mut self, skill: Skill, cost: i32) -> bool {
        let pool = self.pools.get_mut(skill);
        if *pool >= cost {
            *pool -= cost;
            true
        } else {
            false
        }
    }
}