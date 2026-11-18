use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::app::{App, AppLocation, TownLocation, DungeonLocation};
use crate::enemy::Enemy;
use crate::popup::{Popup, PopupKind};

// TODO: replace with the player's weapon/stats once those exist.
const PLAYER_ATTACK_DAMAGE: u32 = 1;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    GoTo(AppLocation),
    /// Flavor text only (placeholder for shops, rumours, etc.).
    Message(&'static str),
    Quit,
    Explore,
    Attack,
    Flee,
    OpenChest,
    LeaveChest,
    DisarmTrap,
    EscapeTrap,
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
const ENTER_DUNGEON: Action = Action::GoTo(AppLocation::Dungeon(DungeonLocation::Exploring));

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

const DUNGEON_EXPLORE: &[Choice] = &[
    choice('x', "E(x)plore", Action::Explore),
    choice('e', "(E)xit", BACK_TO_SQUARE),
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
            DungeonLocation::Entrance => DUNGEON_EXPLORE,
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
        Action::Explore => {
            app.buffer.push("You venture deeper into the dungeon...".to_string());
            start_monster_encounter(app);
        }
        Action::Attack => attack(app),
        Action::Flee => {
            app.buffer.push("You run away from the monster.".to_string());
            end_event(app);
        }
        Action::OpenChest => {
            app.buffer.push("You open the treasure chest!".to_string());
            // For demonstration, let's say you find gold
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
 
    if matches!(to, AppLocation::Dungeon(_)) {
        start_monster_encounter(app);
    }
}
 
fn end_event(app: &mut App) {
    app.location = AppLocation::Dungeon(DungeonLocation::Exploring);
    app.active_enemy = None;
}


fn start_monster_encounter(app: &mut App) {
    // TODO: temporary test spawn; replace with random encounter logic.
    app.active_enemy = Enemy::spawn("translucent_red_slime");
    match &app.active_enemy {
        Some(enemy) => {
            app.buffer.push(format!("A {} appears!", enemy.name()));
            app.location = AppLocation::Dungeon(DungeonLocation::Monster);
        }
        None => {
            app.buffer.push("The dungeon is eerily quiet.".to_string());
            app.location = AppLocation::Dungeon(DungeonLocation::Exploring);
        }
    }
}

fn attack(app: &mut App) {
    let Some(enemy) = app.active_enemy.as_mut() else {
        end_event(app);
        return;
    };
    enemy.take_damage(PLAYER_ATTACK_DAMAGE);
    app.buffer.push(format!(
        "You hit the {} for {} damage! ({}/{} hits left)",
        enemy.name(),
        PLAYER_ATTACK_DAMAGE,
        enemy.current_hits,
        enemy.max_hits(),
    ));
    if enemy.is_dead() {
        app.buffer.push(format!("The {} is defeated!", enemy.name()));
        end_event(app);
    }
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