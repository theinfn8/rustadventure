use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skill {
    Combat,
    Defense,
    Arcana,
    Thievery,
}

impl Skill {
    pub const ALL: [Skill; 4] = [Skill::Combat, Skill::Defense, Skill::Arcana, Skill::Thievery];
}