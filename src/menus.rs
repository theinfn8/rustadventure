use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::app::{App, AppLocation, TownLocation, EventType};
use crate::enemy::Enemy;

// TODO: replace with the player's weapon/stats once those exist.
const PLAYER_ATTACK_DAMAGE: u32 = 1;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    match app.location {
        AppLocation::Town => match app.townlocation {
        Some(TownLocation::TownSquare) => handle_townsquare_key(app, key),
        Some(TownLocation::Guild) => handle_guild_key(app, key),
        Some(TownLocation::Blacksmith) => handle_blacksmith_key(app, key),
        Some(TownLocation::Arcanist) => handle_arcanist_key(app, key),
        Some(TownLocation::Trainer) => handle_trainer_key(app, key),
        None => {}
    },
        AppLocation::Dungeon => handle_dungeon_key(app, key),
    }
}

fn handle_townsquare_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Char('g') => {
            app.buffer.push("You enter the guild.".to_string());
            app.menutext = "(R)umours || (M)essageboard || (E)xit".to_string();
            app.townlocation = Some(TownLocation::Guild);
        }
        KeyCode::Char('b') => {
            app.buffer.push("You enter the blacksmith.".to_string());
            app.menutext = "(B)uy || (S)ell || (E)xit".to_string();
            app.townlocation = Some(TownLocation::Blacksmith);
        }
        KeyCode::Char('a') => {
            app.buffer.push("You enter the arcanist.".to_string());
            app.menutext = "(S)crolls || (P)otions || (M)agic Spells || (E)xit".to_string();
            app.townlocation = Some(TownLocation::Arcanist);
        }
        KeyCode::Char('t') => {
            app.buffer.push("You enter the trainer.".to_string());
            app.menutext = "(T)rain || (E)xit".to_string();
            app.townlocation = Some(TownLocation::Trainer);
        }
        KeyCode::Char('d') => {
            app.buffer.push("You enter the dungeon.".to_string());
            app.townlocation = None;
            app.location = AppLocation::Dungeon;
            start_monster_encounter(app);
        }
        _ => {}
    }
}

fn handle_blacksmith_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('e') => {
            app.buffer.push("You leave the blacksmith and head back to the town square.".to_string());
            app.menutext = "(G)uild || (B)lacksmith || (A)rcanist || (T)rainer || (D)ungeon || (Q)uit".to_string();
            app.townlocation = Some(TownLocation::TownSquare);
        },
        KeyCode::Char('b') => {
            app.buffer.push("You buy some equipment from the blacksmith.".to_string());
        },
        KeyCode::Char('s') => {
            app.buffer.push("You sell some equipment to the blacksmith.".to_string());
        },
        _ => {}
    }
}

fn handle_arcanist_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('e') => {
            app.buffer.push("You leave the arcanist and head back to the town square.".to_string());
            app.menutext = "(G)uild || (B)lacksmith || (A)rcanist || (T)rainer || (D)ungeon || (Q)uit".to_string();
            app.townlocation = Some(TownLocation::TownSquare);
        },
        KeyCode::Char('s') => {
            app.buffer.push("You buy some scrolls from the arcanist.".to_string());
        },
        KeyCode::Char('p') => {
            app.buffer.push("You buy some potions from the arcanist.".to_string());
        },
        KeyCode::Char('m') => {
            app.buffer.push("You learn some magic spells from the arcanist.".to_string());
        },
        _ => {}
    }
}

fn handle_trainer_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('e') => {
            app.buffer.push("You leave the trainer and head back to the town square.".to_string());
            app.menutext = "(G)uild || (B)lacksmith || (A)rcanist || (T)rainer || (D)ungeon || (Q)uit".to_string();
            app.townlocation = Some(TownLocation::TownSquare);
        },
        KeyCode::Char('t') => {
            app.buffer.push("You train with the trainer.".to_string());
        },
        _ => {}
    }
}

fn handle_guild_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('e') => {
            app.buffer.push("You leave the guild and head back to the town square.".to_string());
            app.menutext = "(G)uild || (B)lacksmith || (A)rcanist || (T)rainer || (D)ungeon || (Q)uit".to_string();
            app.townlocation = Some(TownLocation::TownSquare);
        },
        KeyCode::Char('r') => {
            app.buffer.push("You listen in to the local rumours.".to_string());
        },
        KeyCode::Char('m') => {
            app.buffer.push("You read the messages on the messageboard.".to_string());
        },
        _ => {}
    }
}

fn handle_dungeon_key(app: &mut App, key: KeyEvent) {
    // (Q)uit is shown in every dungeon hint bar, so handle it for all states.
    if key.code == KeyCode::Char('q') {
        app.running = false;
        return;
    }

    match app.event {
        Some(EventType::Monster) => handle_monster_key(app, key),
        Some(EventType::Treasure) => handle_treasure_key(app, key),
        Some(EventType::Trap) => handle_trap_key(app, key),
        None => handle_explore_key(app, key),
    }
}

fn handle_explore_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('x') => {
            app.buffer.push("You venture deeper into the dungeon...".to_string());
            start_monster_encounter(app);
        }
        KeyCode::Char('e') => {
            app.buffer.push("You leave the dungeon and head back to the town square.".to_string());
            app.menutext = "(G)uild || (B)lacksmith || (A)rcanist || (T)rainer || (D)ungeon || (Q)uit".to_string();
            app.location = AppLocation::Town;
            app.townlocation = Some(TownLocation::TownSquare);
        }
        _ => {}
    }
}

fn start_monster_encounter(app: &mut App) {
    // TODO: temporary test spawn; replace with random encounter logic.
    app.active_enemy = Enemy::spawn("translucent_red_slime");
    match &app.active_enemy {
        Some(enemy) => {
            app.buffer.push(format!("A {} appears!", enemy.name()));
            app.event = Some(EventType::Monster);
        }
        None => {
            app.buffer.push("The dungeon is eerily quiet.".to_string());
            app.event = None;
        }
    }
}

fn handle_monster_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('a') => {
            let Some(enemy) = app.active_enemy.as_mut() else {
                app.event = None;
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
                app.active_enemy = None;
                app.event = None;
            }
        }
        KeyCode::Char('r') => {
            app.buffer.push("You run away from the monster.".to_string());
            app.active_enemy = None;
            app.event = None;
        }
        _ => {}
    }
}

fn handle_treasure_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('o') => {
            app.buffer.push("You open the treasure chest!".to_string());
            // For demonstration, let's say you find gold
            app.buffer.push("You found 100 gold!".to_string());
            app.event = None;
        }
        KeyCode::Char('l') => {
            app.buffer.push("You leave the treasure chest alone.".to_string());
            app.event = None;
        }
        _ => {}
    }
}

fn handle_trap_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('d') => {
            app.buffer.push("You disarm the trap!".to_string());
            // For demonstration, let's say you successfully disarm it
            app.event = None;
        }
        KeyCode::Char('r') => {
            app.buffer.push("You run away from the trap.".to_string());
            app.event = None;
        }
        _ => {}
    }
}