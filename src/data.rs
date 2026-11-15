use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::sync::OnceLock;

use serde::de::DeserializeOwned;

use crate::item::Item;
use crate::mob::Mob;

pub struct GameData {
    pub mobs: HashMap<String, Mob>,
    pub items: HashMap<String, Item>,
}

static GAME_DATA: OnceLock<GameData> = OnceLock::new();

fn load_list<T: DeserializeOwned>(path: &str) -> Result<Vec<T>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
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

    GAME_DATA
        .set(GameData { items, mobs })
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
