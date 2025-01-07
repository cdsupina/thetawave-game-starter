use bevy::{
    math::Vec2,
    prelude::{Component, Resource},
    utils::hashbrown::HashMap,
};

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
}

/// Component for storing values used in systems for player entities
#[derive(Component)]
pub(super) struct PlayerStatsComponent {
    pub acceleration: f32,
    pub deceleration_factor: f32,
}
