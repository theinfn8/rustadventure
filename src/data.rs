use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::sync::OnceLock;

use serde::de::DeserializeOwned;

use crate::mob::Mob;

pub struct GameData {
    pub mobs: HashMap<String, Mob>,
}

static GAME_DATA: OnceLock<GameData> = OnceLock::new();

fn load_list<T: DeserializeOwned>(path: &str) -> Result<Vec<T>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|e| format!("{path}: {e}").into())
}

pub fn init_game_data() -> Result<(), Box<dyn Error>> {
    let mob_list = load_list::<Mob>("data/mobs.json")?;
    for m in &mob_list {
        m.validate()?;
    }

    let mobs: HashMap<String, Mob> = mob_list
        .into_iter()
        .map(|m| (m.id.clone(), m))
        .collect();

    GAME_DATA
        .set(GameData { mobs })
        .map_err(|_| "Game data already initialized")?;
    Ok(())
}

pub fn get() -> &'static GameData {
    GAME_DATA.get().expect("data::init was not called before data::get ")
}

pub fn mob(id: &str) -> Option<&'static Mob> {
    get().mobs.get(id)
}
