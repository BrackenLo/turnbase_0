//====================================================================

use std::collections::HashMap;

//====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionId(u32);

pub struct ActionRepo {
    action_id: ActionId,
    actions: HashMap<ActionId, Action>,
}

impl ActionRepo {
    pub fn new() -> Self {
        let mut repo = Self {
            action_id: ActionId(0),
            actions: HashMap::default(),
        };

        repo.add_action(Action {
            name: String::from("Idle"),
            target: TargetType::None,
            resolution: ActionResolution::None,
        });

        repo.add_action(Action {
            name: String::from("Punch"),
            target: TargetType::Enemy,
            resolution: ActionResolution::Damage(5),
        });

        repo.add_action(Action {
            name: String::from("Block"),
            target: TargetType::Caster,
            resolution: ActionResolution::Heal(5),
        });

        repo.add_action(Action {
            name: String::from("Heal"),
            target: TargetType::Any {
                can_target_caster: true,
            },
            resolution: ActionResolution::Heal(5),
        });

        repo.add_action(Action {
            name: String::from("Shield"),
            target: TargetType::Friendly {
                can_target_caster: true,
            },
            resolution: ActionResolution::Heal(5),
        });

        repo
    }

    fn add_action(&mut self, action: Action) {
        let id = self.action_id;
        self.action_id.0 += 1;

        self.actions.insert(id, action);
    }

    pub fn find_action_name(&self, name: &str) -> Option<ActionId> {
        match self.actions.iter().find(|(_, action)| action.name == name) {
            Some((id, _)) => Some(*id),
            None => None,
        }
    }

    #[inline]
    pub fn get_action(&self, id: &ActionId) -> Option<&Action> {
        self.actions.get(id)
    }
}

//====================================================================

#[derive(Debug)]
pub struct Action {
    pub name: String,
    pub target: TargetType,
    pub resolution: ActionResolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    None,
    Any { can_target_caster: bool },
    Caster,
    Friendly { can_target_caster: bool },
    Enemy,
}

#[derive(Debug)]
pub enum ActionResolution {
    None,
    Damage(u32),
    Heal(u32),
}

//====================================================================
