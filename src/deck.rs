use std::ops::RangeInclusive;

use rand::seq::{IndexedRandom, SliceRandom};
use rand::Rng;

use crate::boss::{Boss, BossKind};
use crate::data;
use crate::mob::Mob;
use crate::theme::Theme;

// ---- Balance ----

/// Every floor has at least this many rooms...
pub const BASE_ROOMS: u32 = 3;
/// ...plus one more for every this many floors...
pub const FLOORS_PER_EXTRA_ROOM: u32 = 2;
/// ...plus a random number from this range, so players can't count to the end.
pub const EXTRA_ROOMS: RangeInclusive<u32> = 2..=6;
/// Chance that a normal room gets a highlight.
pub const HIGHLIGHT_CHANCE: f64 = 0.6;

/// Relative chance of each event in a normal room.
const EVENT_WEIGHTS: [(EventKind, u32); 4] = [
    (EventKind::Monster, 60),
    (EventKind::Trap, 20),
    (EventKind::Treasure, 10),
    (EventKind::Empty, 10),
];

/// Shown when neither the boss nor the theme has door text.
const DEFAULT_DOOR: &str = "A heavy door blocks the way forward.";
const DEFAULT_STAIRS: &str = "A stairway leads down into the dark.";

/// Fewest normal rooms a floor can have.
pub fn min_rooms(floor: u32) -> u32 {
    BASE_ROOMS + floor / FLOORS_PER_EXTRA_ROOM
}

/// Most normal rooms a floor can have (not counting boss cards).
pub fn max_rooms(floor: u32) -> u32 {
    min_rooms(floor) + EXTRA_ROOMS.end()
}

// ---- Cards ----

/// What happens in a room.
#[derive(Debug, Clone, Copy)]
pub enum RoomEvent {
    Empty,
    Monster(&'static Mob),
    Trap,
    Treasure,
    /// The pause before a boss: the player can enter or retreat.
    BossDoor(&'static Boss),
    /// A boss or mini-boss fight.
    Boss(&'static Boss),
}

/// One card in the deck.
#[derive(Debug, Clone, Copy)]
pub struct Room {
    pub description: &'static str,
    pub highlight: Option<&'static str>,
    pub event: RoomEvent,
}

/// Which kind of event to roll, before choosing the details (e.g. which mob).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventKind {
    Monster,
    Trap,
    Treasure,
    Empty,
}

// ---- Deck ----

/// The rooms of one floor, as a stack: the current room is the last one,
/// and clearing it reveals the next. When it's empty, the stairs appear.
#[derive(Debug, Clone)]
pub struct Deck {
    pub floor: u32,
    pub theme: &'static Theme,
    rooms: Vec<Room>,
}

impl Deck {
    /// Build a fresh, random deck for `floor`.
    pub fn generate(floor: u32, rng: &mut impl Rng) -> Result<Self, String> {
        let theme = data::theme_for_floor(floor)
            .ok_or_else(|| format!("no theme covers floor {floor}"))?;
        let mobs = mobs_for_floor(floor);
        if mobs.is_empty() {
            return Err(format!("no mobs available for floor {floor}"));
        }

        let floor_bosses: Vec<&'static Boss> = data::bosses_for_floor(floor).collect();
        let main_boss = floor_bosses
            .iter()
            .copied()
            .filter(|b| b.kind == BossKind::Boss)
            .collect::<Vec<_>>()
            .choose(rng)
            .copied();
        let minibosses: Vec<&'static Boss> =
            floor_bosses.iter().copied().filter(|b| b.kind == BossKind::Miniboss).collect();

        // Text pools, shuffled so a floor doesn't repeat itself until a pool runs out.
        let mut descriptions = Pool::new(theme.descriptions.iter().map(String::as_str), rng);
        let boss_hints = floor_bosses.iter().flat_map(|b| b.hints.iter().map(String::as_str));
        let mut highlights =
            Pool::new(theme.highlights.iter().map(String::as_str).chain(boss_hints), rng);

        let mut rooms = Vec::new();

        // Bottom of the stack: the boss fight, then its door on top.
        if let Some(boss) = main_boss {
            rooms.push(Room {
                description: &boss.room_description,
                highlight: None,
                event: RoomEvent::Boss(boss),
            });
            let door = boss
                .door_description
                .as_deref()
                .or(theme.boss_door_description.as_deref())
                .unwrap_or(DEFAULT_DOOR);
            rooms.push(Room { description: door, highlight: None, event: RoomEvent::BossDoor(boss) });
        }
        let first_normal = rooms.len();

        // Normal rooms on top.
        let count = min_rooms(floor) + rng.random_range(EXTRA_ROOMS);
        for _ in 0..count {
            let highlight = if rng.random_bool(HIGHLIGHT_CHANCE) { highlights.draw(rng) } else { None };
            rooms.push(Room {
                description: descriptions.draw(rng).unwrap_or(""),
                highlight,
                event: roll_event(&mobs, rng),
            });
        }

        // Mini-bosses are a surprise somewhere among the normal rooms, with no door.
        for boss in minibosses {
            let at = rng.random_range(first_normal..=rooms.len());
            rooms.insert(at, Room {
                description: &boss.room_description,
                highlight: None,
                event: RoomEvent::Boss(boss),
            });
        }

        Ok(Self { floor, theme, rooms })
    }

    /// The room the player is in, or None once the floor is cleared.
    pub fn current(&self) -> Option<&Room> {
        self.rooms.last()
    }

    /// Mutable access to the current room, e.g. to change its event once it's resolved.
    pub fn current_mut(&mut self) -> Option<&mut Room> {
        self.rooms.last_mut()
    }

    /// Discard the current room and move on to the next.
    pub fn clear_room(&mut self) -> Option<Room> {
        self.rooms.pop()
    }

    /// True when every room has been cleared and the stairs are open.
    pub fn is_complete(&self) -> bool {
        self.rooms.is_empty()
    }

    pub fn stairs_description(&self) -> &'static str {
        self.theme.stairs_description.as_deref().unwrap_or(DEFAULT_STAIRS)
    }

    /// Rooms left, for tests and debugging only. Don't show this to the player!
    #[cfg(test)]
    pub fn remaining(&self) -> usize {
        self.rooms.len()
    }
}

// ---- Helpers ----

/// Mobs for a floor: those whose level matches the floor. Deeper than the strongest
/// mobs, the strongest available level is used.
fn mobs_for_floor(floor: u32) -> Vec<&'static Mob> {
    let all = &data::get().mobs;
    let level = all.values().map(|m| m.level).filter(|&l| l <= floor).max();
    match level {
        Some(level) => all.values().filter(|m| m.level == level).collect(),
        None => Vec::new(),
    }
}

fn roll_event(mobs: &[&'static Mob], rng: &mut impl Rng) -> RoomEvent {
    let kind = EVENT_WEIGHTS
        .choose_weighted(rng, |(_, weight)| *weight)
        .map(|(kind, _)| *kind)
        .unwrap_or(EventKind::Empty);
    match kind {
        EventKind::Monster => match mobs.choose(rng) {
            Some(mob) => RoomEvent::Monster(mob),
            None => RoomEvent::Empty,
        },
        EventKind::Trap => RoomEvent::Trap,
        EventKind::Treasure => RoomEvent::Treasure,
        EventKind::Empty => RoomEvent::Empty,
    }
}

/// A shuffled list of text that's drawn in order, reshuffling when it runs out,
/// so nothing repeats until everything has been used once.
struct Pool {
    items: Vec<&'static str>,
    next: usize,
}

impl Pool {
    fn new(items: impl Iterator<Item = &'static str>, rng: &mut impl Rng) -> Self {
        let mut items: Vec<&'static str> = items.collect();
        items.shuffle(rng);
        Self { items, next: 0 }
    }

    fn draw(&mut self, rng: &mut impl Rng) -> Option<&'static str> {
        if self.items.is_empty() {
            return None;
        }
        if self.next == self.items.len() {
            self.items.shuffle(rng);
            self.next = 0;
        }
        let item = self.items[self.next];
        self.next += 1;
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn setup() {
        let _ = data::init(); // already-initialized is fine across tests
    }

    #[test]
    fn room_count_stays_in_range() {
        setup();
        let mut rng = rand::rng();
        for floor in 1..=4 {
            for _ in 0..200 {
                let deck = Deck::generate(floor, &mut rng).unwrap();
                let n = deck.remaining() as u32;
                assert!((min_rooms(floor) + EXTRA_ROOMS.start()..=max_rooms(floor)).contains(&n));
            }
        }
    }

    #[test]
    fn boss_floor_ends_with_door_then_boss() {
        setup();
        let mut rng = rand::rng();
        for _ in 0..100 {
            let mut deck = Deck::generate(5, &mut rng).unwrap();
            let mut last_two = Vec::new();
            while let Some(room) = deck.clear_room() {
                last_two.push(room.event);
                if last_two.len() > 2 {
                    last_two.remove(0);
                }
            }
            assert!(matches!(last_two[0], RoomEvent::BossDoor(_)));
            assert!(matches!(last_two[1], RoomEvent::Boss(_)));
        }
    }

    #[test]
    fn no_boss_on_normal_floors() {
        setup();
        let mut rng = rand::rng();
        let mut deck = Deck::generate(3, &mut rng).unwrap();
        while let Some(room) = deck.clear_room() {
            assert!(!matches!(room.event, RoomEvent::Boss(_) | RoomEvent::BossDoor(_)));
        }
    }

    #[test]
    fn descriptions_dont_repeat_within_a_floor() {
        setup();
        let mut rng = rand::rng();
        for floor in 1..=5 {
            for _ in 0..100 {
                let mut deck = Deck::generate(floor, &mut rng).unwrap();
                let mut seen = HashSet::new();
                while let Some(room) = deck.clear_room() {
                    if matches!(room.event, RoomEvent::Boss(_) | RoomEvent::BossDoor(_)) {
                        continue;
                    }
                    assert!(seen.insert(room.description), "repeated: {}", room.description);
                }
            }
        }
    }

    #[test]
    fn monsters_match_floor_level() {
        setup();
        let mut rng = rand::rng();
        for floor in [1, 3, 7] {
            let mut deck = Deck::generate(floor, &mut rng).unwrap();
            while let Some(room) = deck.clear_room() {
                if let RoomEvent::Monster(mob) = room.event {
                    assert_eq!(mob.level, floor.min(5));
                }
            }
        }
    }
}