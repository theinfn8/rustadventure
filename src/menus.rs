use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::app::{App, AppLocation, TownLocation, DungeonLocation};
use crate::combat::{Affinity, Hit};
use crate::deck::{Deck, RoomEvent};
use crate::dungeon_run::{DungeonRun, RunEnd};
use crate::enemy::Enemy;
use crate::popup::{Popup, PopupKind};


#[derive(Debug, Clone, Copy)]
pub enum Action {
    GoTo(AppLocation),
    /// Flavor text only (placeholder for shops, rumours, etc.).
    Message(&'static str),
    Quit,
    Attack,
    Flee,
    OpenChest,
    LeaveChest,
    DisarmTrap,
    EscapeTrap,
    EnterFloor,
    NextRoom,
    Descend,
    EnterBossRoom,
    LeaveDungeon,
    OpenInventory,
    PopupUp,
    PopupDown,
    PopupSelect,
    PopupClose,
}

pub struct Choice {
    pub key: KeyCode,
    pub label: &'static str,
    pub action: Action,
}

const fn choice(key: char, label: &'static str, action: Action) -> Choice {
    Choice { key: KeyCode::Char(key), label, action }
}

const fn choice_key(key: KeyCode, label: &'static str, action: Action) -> Choice {
    Choice { key, label, action }
}

const fn go_town(loc: TownLocation) -> Action {
    Action::GoTo(AppLocation::Town(loc))
}

const GLOBAL: &[Choice] = &[choice('i', "", Action::OpenInventory)];
const BACK_TO_SQUARE: Action = go_town(TownLocation::TownSquare);
const ENTER_DUNGEON: Action = Action::GoTo(AppLocation::Dungeon(DungeonLocation::Entrance));

const TOWN_SQUARE: &[Choice] = &[
    choice('g', "(G)uild", go_town(TownLocation::Guild)),
    choice('b', "(B)ank", go_town(TownLocation::Bank)),
    choice('s', "(S)mith", go_town(TownLocation::Blacksmith)),
    choice('a', "(A)rcanist", go_town(TownLocation::Arcanist)),
    choice('t', "(T)rainer", go_town(TownLocation::Trainer)),
    choice('d', "(D)ungeon", ENTER_DUNGEON),
    choice('q', "(Q)uit", Action::Quit),
];

const BANK: &[Choice] = &[
    choice('d', "(D)eposit Gold", Action::Message("You deposit your gold.")),
    choice('w', "(W)ithdraw Gold", Action::Message("You withdraw some gold.")),
    choice('e', "(E)xit", BACK_TO_SQUARE),
];

const GUILD: &[Choice] = &[
    choice('r', "(R)umours", Action::Message("You listen in to the local rumours.")),
    choice('m', "(M)essageboard", Action::Message("You read the messages on the messageboard.")),
    choice('s', "(S)leep", Action::Message("You head to your room for the night.")),
    choice('e', "(E)xit", BACK_TO_SQUARE),
];

const BLACKSMITH: &[Choice] = &[
    choice('b', "(B)uy", Action::Message("You buy some equipment from the blacksmith.")),
    choice('s', "(S)ell", Action::Message("You sell some equipment to the blacksmith.")),
    choice('e', "(E)xit", BACK_TO_SQUARE),
];

const ARCANIST: &[Choice] = &[
    choice('s', "(S)crolls", Action::Message("You buy some scrolls from the arcanist.")),
    choice('p', "(P)otions", Action::Message("You buy some potions from the arcanist.")),
    choice('m', "(M)agic Spells", Action::Message("You learn some magic spells from the arcanist.")),
    choice('e', "(E)xit", BACK_TO_SQUARE),
];

const TRAINER: &[Choice] = &[
    choice('t', "(T)rain", Action::Message("You train with the trainer.")),
    choice('e', "(E)xit", BACK_TO_SQUARE),
];

const DUNGEON_ENTRANCE: &[Choice] = &[
    choice('e', "(E)nter Floor 1", Action::EnterFloor),
    choice('l', "(L)eave", BACK_TO_SQUARE),
    choice('q', "(Q)uit", Action::Quit),
];

const DUNGEON_EXPLORE: &[Choice] = &[
    choice('x', "E(x)plore", Action::NextRoom),
    choice('l', "(L)eave Dungeon", Action::LeaveDungeon),
    choice('q', "(Q)uit", Action::Quit),
];

const DUNGEON_BOSS_DOOR: &[Choice] = &[
    choice('e', "(E)nter", Action::EnterBossRoom),
    choice('r', "(R)etreat", Action::LeaveDungeon),
    choice('q', "(Q)uit", Action::Quit),
];

const DUNGEON_STAIRS: &[Choice] = &[
    choice('d', "(D)escend", Action::Descend),
    choice('l', "(L)eave Dungeon", Action::LeaveDungeon),
    choice('q', "(Q)uit", Action::Quit),
];

const DUNGEON_MONSTER: &[Choice] = &[
    choice('a', "(A)ttack", Action::Attack),
    choice('r', "(R)un", Action::Flee),
    choice('q', "(Q)uit", Action::Quit),
];

const DUNGEON_TREASURE: &[Choice] = &[
    choice('o', "(O)pen", Action::OpenChest),
    choice('l', "(L)eave", Action::LeaveChest),
    choice('q', "(Q)uit", Action::Quit),
];

const DUNGEON_TRAP: &[Choice] = &[
    choice('d', "(D)isarm", Action::DisarmTrap),
    choice('r', "(R)un", Action::EscapeTrap),
    choice('q', "(Q)uit", Action::Quit),
];

const POPUP: &[Choice] = &[
    choice_key(KeyCode::Up, "↑↓ Select", Action::PopupUp),
    choice_key(KeyCode::Down, "", Action::PopupDown),
    choice_key(KeyCode::Enter, "Enter Choose", Action::PopupSelect),
    choice_key(KeyCode::Esc, "Esc Close", Action::PopupClose),
];

pub fn choices(app: &App) -> &'static [Choice] {
    if app.popup.is_some() {
        return POPUP;
    }

    match app.location {
        AppLocation::Town(town) => match town {
            TownLocation::TownSquare => TOWN_SQUARE,
            TownLocation::Bank => BANK,
            TownLocation::Guild => GUILD,
            TownLocation::Blacksmith => BLACKSMITH,
            TownLocation::Arcanist => ARCANIST,
            TownLocation::Trainer => TRAINER,
        },
        AppLocation::Dungeon(dungeon) => match dungeon {
            DungeonLocation::Monster => DUNGEON_MONSTER,
            DungeonLocation::Treasure => DUNGEON_TREASURE,
            DungeonLocation::Trap => DUNGEON_TRAP,
            DungeonLocation::Exploring => DUNGEON_EXPLORE,
            DungeonLocation::Entrance => DUNGEON_ENTRANCE,
            DungeonLocation::BossDoor => DUNGEON_BOSS_DOOR,
            DungeonLocation::Stairs => DUNGEON_STAIRS,
        },
    }
}

fn extra_choices(app: &App) -> &'static [Choice] {
    if app.popup.is_some() { &[] } else { GLOBAL }
}

pub fn hint_text(app: &App) -> String {
    choices(app)
        .iter()
        .chain(extra_choices(app))
        .map(|c| c.label)
        .filter(|label| !label.is_empty())
        .collect::<Vec<_>>()
        .join("|")
}


pub fn handle_key(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    let code = match key.code {
        KeyCode::Char(c) => KeyCode::Char(c.to_ascii_lowercase()),
        other => other,
    };

    let found = choices(app)
        .iter()
        .chain(extra_choices(app))
        .find(|ch| ch.key == code);

    if let Some(choice) = found {
        apply(app, choice.action);
    }

}

fn apply(app: &mut App, action: Action) {
    match action {
        Action::GoTo(to) => go_to(app, to),
        Action::Message(text) => app.buffer.push(text.to_string()),
        Action::Quit => app.running = false,
        Action::EnterFloor => start_floor(app, 1),
        Action::NextRoom => {
            if let Some(run) = app.run.as_mut() {
                run.deck.clear_room();
            }
            enter_room(app);
        }
        Action::Descend => {
            let next = app.run.as_ref().map_or(1, |run| run.floor() + 1);
            app.buffer.push("You descend the stairs...".to_string());
            start_floor(app, next);
        }
        Action::EnterBossRoom => {
            if let Some(run) = app.run.as_mut() {
                run.deck.clear_room(); // the door
            }
            enter_room(app);
        }
        Action::LeaveDungeon => {
            app.buffer.push("You make your way back to the surface.".to_string());
            end_run(app, RunEnd::Left);
        }
        Action::Attack => attack(app),
        Action::Flee => {
            // Running ends the run: there's no slipping past a monster to the next room.
            app.buffer.push("You flee back toward the surface!".to_string());
            end_run(app, RunEnd::Fled);
        }
        Action::OpenChest => {
            app.buffer.push("You open the treasure chest!".to_string());
            // TODO: roll on the treasure tables. For now, a fixed amount into the run's pouch.
            if let Some(run) = app.run.as_mut() {
                run.gold_found += 100;
            }
            app.buffer.push("You found 100 gold!".to_string());
            end_event(app);
        }
        Action::LeaveChest => {
            app.buffer.push("You leave the treasure chest alone.".to_string());
            end_event(app);
        }
        Action::DisarmTrap => {
            app.buffer.push("You disarm the trap!".to_string());
            end_event(app);
        }
        Action::EscapeTrap => {
            app.buffer.push("You run away from the trap.".to_string());
            end_event(app);
        }
        Action::OpenInventory => open_inventory(app),
        Action::PopupUp => {
            if let Some(popup) = app.popup.as_mut() {
                popup.previous();
            }
        }
        Action::PopupDown => {
            if let Some(popup) = app.popup.as_mut() {
                popup.next();
            }
        }
        Action::PopupSelect => {
            if let Some(popup) = app.popup.take() {
                on_popup_select(app, popup.kind, popup.selected());
            }
        }
        Action::PopupClose => app.popup = None,
    }
}

fn go_to(app: &mut App, to: AppLocation) {
    let from = app.location;
    let text = if to == AppLocation::Town(TownLocation::TownSquare) {
        format!("You leave the {} and head back to the town square.", from.name())
    } else {
        format!("You enter the {}.", to.name())
    };
    app.buffer.push(text);
    if let Some(desc) = to.description() {
        app.buffer.push(desc.to_string());
    }
    app.location = to;
    app.active_enemy = None;

    // Leaving the dungeon by any route ends the run; the next visit starts fresh.
    if !matches!(to, AppLocation::Dungeon(_)) {
        app.run = None;
    }
}
 
fn end_event(app: &mut App) {
    app.location = AppLocation::Dungeon(DungeonLocation::Exploring);
    app.active_enemy = None;
}


fn attack(app: &mut App) {
    let Some(enemy) = app.active_enemy.as_mut() else {
        end_event(app);
        return;
    };

    // Still recovering from the last swing: ignore the key press.
    if app.attack_cooldown > 0 {
        return;
    }
    let Some(enemy) = app.active_enemy.as_mut() else {
        end_event(app);
        return;
    };

    let rate = app.character.attack_rate();
    app.attack_cooldown = rate;
    app.attack_cooldown_max = rate;
    
    let hits = enemy.take_hits(&app.character.attack_damage());
    let total: u32 = hits.iter().map(|h| h.dealt).sum();
    app.buffer.push(format!(
        "You hit the {} for {} damage! ({}/{} hits left)",
        enemy.name(),
        total,
        enemy.current_hits,
        enemy.max_hits(),
    ));
    // Only the interesting parts get their own line, so normal hits stay short.
    for hit in &hits {
        if let Some(line) = affinity_message(enemy.name(), hit) {
            app.buffer.push(line);
        }
    }

    if enemy.is_dead() {
        // Bosses get their own defeat text; everything else gets the generic line.
        let boss_text = match app.run.as_ref().and_then(|run| run.deck.current()).map(|r| r.event) {
            Some(RoomEvent::Boss(boss)) => Some(boss.defeat_text.as_str()),
            _ => None,
        };
        match boss_text {
            Some(text) => app.buffer.push(text.to_string()),
            None => app.buffer.push(format!("The {} is defeated!", enemy.name())),
        }
        end_event(app);
    }
}

/// A combat log line explaining an affinity, or None for a normal hit.
fn affinity_message(name: &str, hit: &Hit) -> Option<String> {
    let kind = hit.damage.damage_type.to_string().to_lowercase();
    match hit.affinity? {
        Affinity::Immune => Some(format!("The {name} is unaffected by the {kind} damage!")),
        Affinity::Resistant => Some(format!("The {name} resists the {kind} damage.")),
        Affinity::Weak => Some(format!("The {kind} damage strikes a weak spot!")),
    }
}

// ---- Dungeon floors ----

const BACK_TO_TOWN: AppLocation = AppLocation::Town(TownLocation::TownSquare);

/// Finish the current run and return to town. A safe exit or a successful escape keeps
/// what was found; death loses it.
fn end_run(app: &mut App, how: RunEnd) {
    if let Some(run) = app.run.take() {
        match how {
            RunEnd::Left | RunEnd::Fled => {
                if run.gold_found > 0 {
                    app.character.gold += run.gold_found;
                    app.buffer.push(format!("You bring {} gold back to town.", run.gold_found));
                }
            }
            RunEnd::Died => {
                if run.gold_found > 0 {
                    app.buffer.push(format!("The {} gold you found is lost.", run.gold_found));
                }
            }
        }
    }
    go_to(app, BACK_TO_TOWN);
}

/// Generate a new deck for `floor` and show its first room.
/// Starts a new run if there isn't one; otherwise the run continues onto the new floor.
fn start_floor(app: &mut App, floor: u32) {
    match Deck::generate(floor, &mut rand::rng()) {
        Ok(deck) => {
            app.buffer.push(format!("Floor {floor}: {}", deck.theme.name));
            match app.run.as_mut() {
                Some(run) => run.next_floor(deck),
                None => app.run = Some(DungeonRun::new(deck, app.character.total_skills())),
            }
            enter_room(app);
        }
        Err(e) => {
            app.buffer.push(format!("The way down is blocked. ({e})"));
            app.location = AppLocation::Dungeon(DungeonLocation::Entrance);
        }
    }
}

/// Describe the current room and start its event. With no rooms left, show the stairs.
fn enter_room(app: &mut App) {
    let Some(deck) = app.run.as_ref().map(|run| &run.deck) else {
        app.location = AppLocation::Dungeon(DungeonLocation::Entrance);
        return;
    };

    let Some(room) = deck.current().copied() else {
        // Floor cleared: the stairs appear, and the next floor becomes available.
        let floor = deck.floor;
        app.buffer.push(deck.stairs_description().to_string());
        app.character.dungeon_level = app.character.dungeon_level.max(floor + 1);
        app.location = AppLocation::Dungeon(DungeonLocation::Stairs);
        return;
    };

    app.buffer.push(room.description.to_string());
    if let Some(highlight) = room.highlight {
        app.buffer.push(highlight.to_string());
    }

    let dungeon = |loc| AppLocation::Dungeon(loc);
    app.location = match room.event {
        RoomEvent::Empty => dungeon(DungeonLocation::Exploring),
        RoomEvent::Monster(mob) => {
            app.active_enemy = Some(Enemy::new(mob));
            app.buffer.push(format!("A {} appears!", mob.name));
            dungeon(DungeonLocation::Monster)
        }
        RoomEvent::Trap => dungeon(DungeonLocation::Trap),
        RoomEvent::Treasure => dungeon(DungeonLocation::Treasure),
        RoomEvent::BossDoor(_) => dungeon(DungeonLocation::BossDoor),
        RoomEvent::Boss(boss) => {
            app.active_enemy = Some(Enemy::new(&boss.mob));
            app.buffer.push(format!("The {} rises to face you!", boss.mob.name));
            dungeon(DungeonLocation::Monster)
        }
    };
}

fn open_inventory(app: &mut App) {
    let items = app.character.inventory.iter().map(|e| e.label()).collect();
    app.popup = Some(Popup::new(PopupKind::Inventory, "Inventory", items));
}

/// The player picked entry `index` from a popup (None if the list was empty).
fn on_popup_select(app: &mut App, kind: PopupKind, index: Option<usize>) {
    let Some(index) = index else { return };
    match kind {
        PopupKind::Inventory => {
            // Placeholder: show what the item is. Later: use, drop, equip...
            if let Some(entry) = app.character.inventory.get(index) {
                let item = entry.template();
                app.buffer.push(format!("{}: {}", item.name, item.codex));
            }
        }
        PopupKind::Attacks | PopupKind::Defenses => {
            // TODO: assign or use the chosen ability once abilities exist.
        }
    }
}