use serde::{Deserialize, Serialize};

/// The look and feel of a stretch of floors, loaded from themes.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub min_floor: u32,
    pub max_floor: u32,
    /// What a room looks like. One is drawn for each room card.
    pub descriptions: Vec<String>,
    /// Something extra that catches the player's attention. Not every room gets one.
    #[serde(default)]
    pub highlights: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stairs_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss_door_description: Option<String>,
}

impl Theme {
    pub fn covers(&self, floor: u32) -> bool {
        (self.min_floor..=self.max_floor).contains(&floor)
    }
}