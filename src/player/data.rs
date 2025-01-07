use crate::input::PlayerAbilities;
use bevy::{
    math::Vec2,
    prelude::{Component, Resource},
    utils::hashbrown::HashMap,
};
use leafwing_abilities::prelude::CooldownState;

/// Resource for storing all of the data about every character
#[derive(Resource)]
pub(super) struct CharactersResource {
    pub characters: HashMap<String, CharacterData>,
}

impl Default for CharactersResource {
    fn default() -> Self {
        Self {
            characters: [(
                "captain".to_string(),
                CharacterData {
                    acceleration: 2.0,
                    deceleration_factor: 0.972,
                    max_speed: 100.0,
                    collider_dimensions: Vec2::new(6.0, 12.0),
                    cooldowns: CooldownState::<PlayerAbilities>::new(
                        [
                            (PlayerAbilities::BasicAttack, 0.5),
                            (PlayerAbilities::SecondaryAttack, 1.5),
                            (PlayerAbilities::Utility, 2.0),
                            (PlayerAbilities::Ultimate, 10.0),
                        ]
                        .into(),
                    ),
                },
            )]
            .into(),
        }
    }
}

/// All data used to construct a player entity
pub(super) struct CharacterData {
    pub acceleration: f32,
    pub deceleration_factor: f32,
    pub max_speed: f32,
    pub collider_dimensions: Vec2,
    pub cooldowns: CooldownState<PlayerAbilities>,
}

/// Component for storing values used in systems for player entities
#[derive(Component)]
pub(super) struct PlayerStats {
    pub acceleration: f32,
    pub deceleration_factor: f32,
}
