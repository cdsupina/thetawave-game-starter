use bevy::{ecs::resource::Resource, math::Vec2, platform::collections::HashMap};
use leafwing_abilities::prelude::{Cooldown, CooldownState};
use serde::Deserialize;

use crate::input::{InputType, PlayerAbility, PlayerNum};

/// Resource for storing all of the data about every character
#[derive(Resource, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CharactersResource {
    pub characters: HashMap<String, CharacterAttributes>,
}

const DEFAULT_INHERITED_VELOCITY_MULTIPLIER: f32 = 0.5;

/// Attributes of a character
#[derive(Debug)]
pub struct CharacterAttributes {
    pub acceleration: f32,
    pub deceleration_factor: f32,
    pub max_speed: f32,
    pub collider_dimensions: Vec2,
    pub cooldowns: CooldownState<PlayerAbility>,
    pub restitution: f32,
    pub health: u32,
    pub projectile_damage: u32,
    pub projectile_speed: f32,
    pub projectile_range_seconds: f32,
    pub inherited_velocity_multiplier: f32,
    pub projectile_spawner_position: Vec2,
    pub abilities: HashMap<PlayerAbility, String>,
}

impl<'de> Deserialize<'de> for CharacterAttributes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Define a "helper" struct that mirrors CharacterAttributes
        // but uses types that can be deserialized
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Helper {
            acceleration: f32,
            deceleration_factor: f32,
            max_speed: f32,
            collider_dimensions: Vec2,
            // Instead of CooldownState, use a HashMap with seconds as f32
            cooldowns: HashMap<PlayerAbility, f32>,
            restitution: f32,
            health: u32,
            projectile_damage: u32,
            projectile_speed: f32,
            projectile_range_seconds: f32,
            #[serde(default = "default_inherited_velocity_multiplier")]
            inherited_velocity_multiplier: f32,
            projectile_spawner_position: Vec2,
            abilities: HashMap<PlayerAbility, String>,
        }

        fn default_inherited_velocity_multiplier() -> f32 {
            DEFAULT_INHERITED_VELOCITY_MULTIPLIER
        }

        // Let serde deserialize into the Helper struct first
        let helper = Helper::deserialize(deserializer)?;

        // Transform the deserialized data into the format we need
        let cooldown_pairs: Vec<(PlayerAbility, Cooldown)> = helper
            .cooldowns
            .into_iter()
            .map(|(ability, secs)| (ability, Cooldown::from_secs(secs)))
            .collect();

        // Construct our actual struct with the transformed data
        Ok(CharacterAttributes {
            acceleration: helper.acceleration,
            deceleration_factor: helper.deceleration_factor,
            max_speed: helper.max_speed,
            collider_dimensions: helper.collider_dimensions,
            // Create CooldownState from the transformed pairs
            cooldowns: CooldownState::new(cooldown_pairs),
            restitution: helper.restitution,
            health: helper.health,
            projectile_damage: helper.projectile_damage,
            projectile_speed: helper.projectile_speed,
            projectile_range_seconds: helper.projectile_range_seconds,
            inherited_velocity_multiplier: helper.inherited_velocity_multiplier,
            projectile_spawner_position: helper.projectile_spawner_position,
            abilities: helper.abilities,
        })
    }
}

/// Resource for transferring character choices from character selection screen to game
#[derive(Resource, Default, Debug)]
pub struct ChosenCharactersResource {
    pub players: HashMap<PlayerNum, ChosenCharacterData>,
}

impl ChosenCharactersResource {
    pub fn contains_input(&self, input_type: InputType) -> bool {
        for (_, data) in self.players.iter() {
            if data.input == input_type {
                return true;
            }
        }
        false
    }

    /// Finds the next available PlayerNum (not yet in ChosenCharactersResource)
    pub fn next_available_player_num(&self) -> Option<PlayerNum> {
        let max_used_player_num = self.players.keys().max().cloned();

        if let Some(player_num) = max_used_player_num {
            player_num.next()
        } else {
            None
        }
    }
}

/// Resource for transferring character choices from character selection screen to game
#[derive(Clone, Debug)]
pub struct ChosenCharacterData {
    pub character: String,
    pub input: InputType,
}
