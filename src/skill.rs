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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Skills {
    pub combat: i32,
    pub defense: i32,
    pub arcana: i32,
    pub thievery: i32,
}

impl Skills {
    pub fn get_mut(&mut self, skill: Skill) -> &mut i32 {
        match skill {
            Skill::Combat => &mut self.combat,
            Skill::Defense => &mut self.defense,
            Skill::Arcana => &mut self.arcana,
            Skill::Thievery => &mut self.thievery,
        }
    }

    pub const ALL: [Skill; 4] = [
        Skill::Combat,
        Skill::Defense,
        Skill::Arcana,
        Skill::Thievery
    ];
}