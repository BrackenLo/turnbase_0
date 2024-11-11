//====================================================================

use std::collections::{HashMap, VecDeque};

use super::characters::Character;

//====================================================================

pub struct BattleServer {
    current_character: CharacterId,
    turn_order: VecDeque<CharacterId>,
}

//====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CharacterId(u32);

pub struct CharacterStorage {
    characters: HashMap<CharacterId, Character>,
}

//====================================================================
