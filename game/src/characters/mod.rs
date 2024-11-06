//====================================================================

use actions::Action;

pub mod actions;

//====================================================================

pub struct CharacterId(u32);

pub struct CharacterSpawner {
    current_id: CharacterId,
}

pub struct CharacterManager {
    friendly: Vec<CharacterId>,
    enemy: Vec<CharacterId>,
}

//====================================================================

pub struct Character {
    name: String,
    player_controlled: bool,
    stats: CharacterStats,
    actions: Vec<Action>,
}

pub struct CharacterStats {
    speed: u32,
}

//====================================================================
