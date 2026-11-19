use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::sync::OnceLock;

use serde::de::DeserializeOwned;

use crate::item::Item;
use crate::mob::Mob;
use crate::boss::Boss;
use crate::deck;
use crate::theme::Theme;

pub struct GameData {
    pub mobs: HashMap<String, Mob>,
    pub items: HashMap<String, Item>,
    pub bosses: Vec<Boss>,
    pub themes: Vec<Theme>,
    
}

static GAME_DATA: OnceLock<GameData> = OnceLock::new();

fn load_list<T: DeserializeOwned>(path: &str) -> Result<Vec<T>, Box<dyn Error>> {
    let text = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("{path}: {e}").into())
}

pub fn init_game_data() -> Result<(), Box<dyn Error>> {
    let item_list = load_list::<Item>("data/items.json")?;
    let items = index_by_id(item_list, |i| &i.id, "data/items.json")?;

    let mob_list = load_list::<Mob>("data/mobs.json")?;
    for m in &mob_list {
        m.validate()?;
    }

    let mobs = index_by_id(mob_list, |m| &m.id, "data/mobs.json")?;

    let bosses = load_list::<Boss>("data/bosses.json")?;
    for b in &bosses {
        b.mob.validate()?;
        if mobs.contains_key(&b.mob.id) {
            return Err(format!("data/bosses.json: '{}' is also in mobs.json", b.mob.id).into());
        }
    }

    let themes = load_list::<Theme>("data/themes.json")?;
    validate_themes(&themes)?;

    GAME_DATA
        .set(GameData { items, mobs, bosses, themes })
        .map_err(|_| "Game data already initialized")?;
    Ok(())
}

/// Turn a Vec into a HashMap keyed by ID, rejecting duplicate IDs.
fn index_by_id<T>(
    list: Vec<T>,
    id: impl Fn(&T) -> &str,
    path: &str,
) -> Result<HashMap<String, T>, Box<dyn Error>> {
    let mut map = HashMap::new();
    for entry in list {
        let key = id(&entry).to_string();
        if map.insert(key.clone(), entry).is_some() {
            return Err(format!("{path}: duplicate id '{key}'").into());
        }
    }
    Ok(map)
}

pub fn get() -> &'static GameData {
    GAME_DATA.get().expect("data::init was not called")
}

pub fn item(id: &str) -> Option<&'static Item> {
    get().items.get(id)
}

pub fn mob(id: &str) -> Option<&'static Mob> {
    get().mobs.get(id)
}

pub fn bosses_for_floor(floor: u32) -> impl Iterator<Item = &'static Boss> {
    get().bosses.iter().filter(move |b| b.floor == floor)
}
 
/// The theme for a floor. Floors deeper than every theme reuse the deepest one.
pub fn theme_for_floor(floor: u32) -> Option<&'static Theme> {
    let themes = &get().themes;
    themes
        .iter()
        .find(|t| t.covers(floor))
        .or_else(|| themes.iter().max_by_key(|t| t.max_floor))
}
 
/// Every floor from 1 to the deepest theme must be covered by exactly one theme,
/// and each theme needs enough descriptions that its floors don't repeat a room.
fn validate_themes(themes: &[Theme]) -> Result<(), Box<dyn Error>> {
    const PATH: &str = "data/themes.json";
    let deepest = themes.iter().map(|t| t.max_floor).max().ok_or(format!("{PATH}: no themes"))?;
    for floor in 1..=deepest {
        let count = themes.iter().filter(|t| t.covers(floor)).count();
        if count != 1 {
            return Err(format!("{PATH}: floor {floor} is covered by {count} themes (needs exactly 1)").into());
        }
    }
    for t in themes {
        if t.min_floor > t.max_floor {
            return Err(format!("{PATH}: '{}' has min_floor above max_floor", t.id).into());
        }
        let needed = deck::max_rooms(t.max_floor) as usize;
        if t.descriptions.len() < needed {
            return Err(format!(
                "{PATH}: '{}' has {} descriptions but floor {} can have {} rooms",
                t.id, t.descriptions.len(), t.max_floor, needed
            )
            .into());
        }
    }
    Ok(())
}
