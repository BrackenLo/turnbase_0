#![allow(dead_code)]
//====================================================================

//====================================================================

#[derive(Debug)]
pub struct Action {
    name: String,
    target: TargetType,
    can_target_caster: bool,
    resolution: ActionResolution,
}

#[derive(Debug)]
pub enum TargetType {
    None,
    Any,
    Caster,
    Friendly,
    Enemey,
}

#[derive(Debug)]
pub struct ActionResolution {}

//====================================================================
