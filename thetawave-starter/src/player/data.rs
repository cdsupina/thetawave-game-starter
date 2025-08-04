use crate::input::{InputType, PlayerAbility};
use bevy::{
    math::Vec2,
    prelude::{Component, Resource},
};
use bevy_platform::collections::HashMap;
use leafwing_abilities::prelude::{Cooldown, CooldownState};
use serde::Deserialize;
use strum_macros::{AsRefStr, EnumIter};

/// Resource for storing all of the data about every character
#[derive(Resource, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(super) struct CharactersResource {
    pub characters: HashMap<CharacterType, CharacterAttributes>,
}

/// Characters that can be chosen by players to play as
#[derive(Eq, PartialEq, Hash, Debug, EnumIter, Clone, Deserialize)]
pub(crate) enum CharacterType {
    Captain,
    Juggernaut,
    Doomwing,
}

/// Attributes of a character
#[derive(Debug)]
pub(super) struct CharacterAttributes {
    pub acceleration: f32,
    pub deceleration_factor: f32,
    pub max_speed: f32,
    pub collider_dimensions: Vec2,
    pub cooldowns: CooldownState<PlayerAbility>,
    pub restitution: f32,
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
        })
    }
}

/// Component for storing values used in systems for player entities
#[derive(Component)]
pub(super) struct PlayerStats {
    pub acceleration: f32,
    pub deceleration_factor: f32,
}

/// Tag for indicating multiplayer association
#[derive(Component, Debug, Clone, PartialEq, Eq, AsRefStr, Hash, PartialOrd, Ord)]
pub(crate) enum PlayerNum {
    One,
    Two,
    Three,
    Four,
}

impl PlayerNum {
    pub(crate) fn next(&self) -> Option<Self> {
        match self {
            PlayerNum::One => Some(Self::Two),
            PlayerNum::Two => Some(Self::Three),
            PlayerNum::Three => Some(Self::Four),
            PlayerNum::Four => None,
        }
    }
}

impl TryFrom<&String> for PlayerNum {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "player_one" => Ok(Self::One),
            "player_two" => Ok(Self::Two),
            "player_three" => Ok(Self::Three),
            "player_four" => Ok(Self::Four),
            _ => Err("Invalid player".to_string()),
        }
    }
}

/// Resource for transferring character choices from character selection screen to game
#[derive(Resource, Default, Debug)]
pub(crate) struct ChosenCharactersResource {
    pub players: HashMap<PlayerNum, ChosenCharacterData>,
}

impl ChosenCharactersResource {
    pub(crate) fn contains_input(&self, input_type: InputType) -> bool {
        for (_, data) in self.players.iter() {
            if data.input == input_type {
                return true;
            }
        }
        false
    }

    /// Finds the next available PlayerNum (not yet in ChosenCharactersResource)
    pub(crate) fn next_available_player_num(&self) -> Option<PlayerNum> {
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
pub(crate) struct ChosenCharacterData {
    pub character: CharacterType,
    pub input: InputType,
}
