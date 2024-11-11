//====================================================================

use std::collections::HashSet;

use common::Transform;
use engine::{tools::KeyCode, StateInner};
use hecs::{Entity, World};
use renderer::pipelines::ui3d_pipeline::Ui3d;

use super::{
    characters::{
        actions::{Action, ActionRepo, TargetType},
        Character,
    },
    Characters,
};

//====================================================================

#[derive(Debug)]
pub struct UiMenus {
    action_menu: Entity,
    target_menu: Option<Entity>,

    current_character: Entity,
}

enum UiMenuAction {
    Back,
    Forward,
    Select,
}

pub enum UiMenuOutput {
    None,
    SkipTurn,
}

impl UiMenus {
    pub fn new(
        state: &mut StateInner,
        actions: &ActionRepo,
        current_character: Entity,
    ) -> Result<Self, ()> {
        let menu_pos = {
            let character_transform = state.world.get::<&Transform>(current_character).unwrap();
            character_transform.translation + character_transform.right() * 50.
        };

        let character_actions = state
            .world
            .get::<&Character>(current_character)
            .unwrap()
            .actions
            .iter()
            .map(|action| actions.get_action(action).unwrap().name.clone())
            .collect::<Vec<_>>();

        if character_actions.is_empty() {
            return Err(());
        }

        let action_menu = state.world.spawn((
            Ui3d {
                options: character_actions,
                ..Default::default()
            },
            Transform::from_scale_translation((0.8, 0.8, 0.8), menu_pos),
        ));

        Ok(Self {
            action_menu,
            target_menu: None,
            current_character,
        })
    }

    fn spawn_target_menu(
        &mut self,
        world: &mut World,
        characters: &Characters,
        action: &Action,
    ) -> Result<(), ()> {
        let friendly = characters.friendly.contains(&self.current_character);

        let options = match (action.target, friendly) {
            (TargetType::Any { can_target_caster }, _) => {
                let mut characters = characters
                    .friendly()
                    .iter()
                    .chain(characters.enemy())
                    .map(|id| *id)
                    .collect::<HashSet<_>>();

                if !can_target_caster {
                    characters.remove(&self.current_character);
                }

                characters
            }

            (TargetType::Friendly { can_target_caster }, true) => {
                let mut characters = characters.friendly().clone();
                if !can_target_caster {
                    characters.remove(&self.current_character);
                }
                characters
            }
            (TargetType::Friendly { can_target_caster }, false) => {
                let mut characters = characters.enemy().clone();
                if !can_target_caster {
                    characters.remove(&self.current_character);
                }
                characters
            }

            (TargetType::Enemy, true) => characters.friendly().clone(),
            (TargetType::Enemy, false) => characters.enemy().clone(),

            _ => todo!(),
        };

        if options.is_empty() {
            return Err(());
        }

        let options = options
            .into_iter()
            .map(|id| world.get::<&Character>(id).unwrap().name.clone())
            .collect::<Vec<_>>();

        self.target_menu = world
            .spawn((
                Transform::from_scale((0.3, 0.3, 0.3)),
                Ui3d {
                    options,
                    ..Default::default()
                },
            ))
            .into();

        Ok(())
    }

    pub fn drop_menus(&self, world: &mut World) {
        world.despawn(self.action_menu).ok();
        if let Some(target_menu) = self.target_menu {
            world.despawn(target_menu).ok();
        }
    }

    pub fn tick(
        &mut self,
        state: &mut StateInner,
        action_repo: &ActionRepo,
        characters: &Characters,
    ) -> UiMenuOutput {
        self.position_children(state);

        // Process target menu if available
        if let Some(target_menu) = self.target_menu {
            match Self::process_input(state, target_menu) {
                Some(UiMenuAction::Forward | UiMenuAction::Select) => {
                    return UiMenuOutput::SkipTurn;
                }
                Some(UiMenuAction::Back) => {
                    state.world.despawn(target_menu).ok();
                    self.target_menu = None;
                }
                None => {}
            }

            return UiMenuOutput::None;
        }

        // Process Actions menu
        match Self::process_input(state, self.action_menu) {
            // Forward or select entered
            Some(UiMenuAction::Forward | UiMenuAction::Select) => {
                println!("Seledted to dosthings");
                let action = {
                    let ui = state.world.get::<&Ui3d>(self.action_menu).unwrap();
                    let character = state
                        .world
                        .get::<&Character>(self.current_character)
                        .unwrap();

                    *character.actions.get(ui.selected as usize).unwrap()
                };

                let action = action_repo.get_action(&action).unwrap();

                match action.target {
                    TargetType::None | TargetType::Caster => return UiMenuOutput::SkipTurn,
                    _ => {
                        self.spawn_target_menu(&mut state.world, characters, &action)
                            .ok();
                        self.position_children(state);
                    }
                }
            }
            // Don't care about anything else
            _ => {}
        }

        UiMenuOutput::None
    }

    fn position_children(&mut self, state: &mut StateInner) {
        if let Some(target_menu) = self.target_menu {
            let new_pos = {
                let parent_transform = state.world.get::<&Transform>(self.action_menu).unwrap();

                parent_transform.translation
                    + parent_transform.right() * (parent_transform.scale.x * 100.)
                    + parent_transform.forward() * 2.
            };

            let mut transform = state.world.get::<&mut Transform>(target_menu).unwrap();

            transform.translation = new_pos;
        }
    }

    fn process_input(state: &mut StateInner, target: Entity) -> Option<UiMenuAction> {
        let keys = &mut state.keys;

        let up_pressed = keys.just_pressed(KeyCode::ArrowUp);
        let down_pressed = keys.just_pressed(KeyCode::ArrowDown);
        let dir = down_pressed as i8 - up_pressed as i8;

        let action = if keys.just_pressed(KeyCode::Enter) {
            Some(UiMenuAction::Select)
        } else if keys.just_pressed(KeyCode::ArrowRight) {
            Some(UiMenuAction::Forward)
        } else if keys.just_pressed(KeyCode::ArrowLeft) {
            Some(UiMenuAction::Back)
        } else {
            None
        };

        let mut ui = state.world.get::<&mut Ui3d>(target).unwrap();

        let selected = ui.selected as i8 + dir;
        ui.selected = selected.clamp(0, ui.options.len() as i8 - 1) as u8;

        return action;
    }
}

//====================================================================
