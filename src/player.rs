
pub struct Player {
    pub name: String,
    pub level: u32,
    pub xp: u32,
    pub max_hits: u32,
    pub skills: vec<Skill>,
    pub gold: u32,
    pub gear: hashmap<String, Item>,
}