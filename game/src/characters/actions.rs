#![allow(dead_code)]
//====================================================================

//====================================================================

pub struct Action {
    name: String,
    target: TargetType,
    can_target_caster: bool,
    resolution: ActionResolution,
}

pub enum TargetType {
    None,
    Any,
    Caster,
    Friendly,
    Enemey,
}

pub struct ActionResolution {}

//====================================================================
