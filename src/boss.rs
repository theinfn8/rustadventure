use serde::{Deserialize, Serialize};
use crate::mob::Mob;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BossKind {
    Boss,
    Miniboss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boss {
    pub kind: BossKind,
    pub floor: u32,
    pub room_description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub door_description: Option<String>,
    pub defeat_text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<String>,
    pub mob: Mob,
}