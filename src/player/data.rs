use crate::input::PlayerAbilities;
use bevy::{
    math::Vec2,
    prelude::{Component, Resource},
    utils::hashbrown::HashMap,
};
use leafwing_abilities::prelude::{Cooldown, CooldownState};

/// Resource for storing all of the data about every character
#[derive(Resource)]
pub(super) struct CharactersResource {
    pub characters: HashMap<String, CharacterData>,
}

impl Default for CharactersResource {
    fn default() -> Self {
        let mut captain_cooldowns = CooldownState::default();

        captain_cooldowns.set(PlayerAbilities::BasicAttack, Cooldown::from_secs(0.5));
        captain_cooldowns.set(PlayerAbilities::SecondaryAttack, Cooldown::from_secs(1.5));
        captain_cooldowns.set(PlayerAbilities::Utility, Cooldown::from_secs(2.0));
        captain_cooldowns.set(PlayerAbilities::Ultimate, Cooldown::from_secs(10.0));

        Self {
            characters: [(
                "captain".to_string(),
                CharacterData {
                    acceleration: 2.0,
                    deceleration_factor: 0.972,
                    max_speed: 100.0,
                    collider_dimensions: Vec2::new(6.0, 12.0),
                    cooldowns: captain_cooldowns,
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
