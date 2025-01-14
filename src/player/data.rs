use crate::input::PlayerAbilities;
use bevy::{
    math::Vec2,
    prelude::{Component, Event, Resource},
    utils::hashbrown::HashMap,
};
use leafwing_abilities::prelude::{Cooldown, CooldownState};
use strum::{AsRefStr, EnumIter};

/// Resource for storing all of the data about every character
#[derive(Resource)]
pub(super) struct CharactersResource {
    pub characters: HashMap<CharacterType, CharacterData>,
}

impl Default for CharactersResource {
    fn default() -> Self {
        Self {
            characters: [
                (
                    CharacterType::Captain,
                    CharacterData {
                        acceleration: 2.0,
                        deceleration_factor: 0.972,
                        max_speed: 100.0,
                        collider_dimensions: Vec2::new(6.0, 12.0),
                        cooldowns: CooldownState::<PlayerAbilities>::new([
                            (PlayerAbilities::BasicAttack, Cooldown::from_secs(0.5)),
                            (PlayerAbilities::SecondaryAttack, Cooldown::from_secs(1.5)),
                            (PlayerAbilities::Utility, Cooldown::from_secs(2.0)),
                            (PlayerAbilities::Ultimate, Cooldown::from_secs(10.0)),
                        ]),
                    },
                ),
                (
                    CharacterType::Juggernaut,
                    CharacterData {
                        acceleration: 1.8,
                        deceleration_factor: 0.988,
                        max_speed: 90.0,
                        collider_dimensions: Vec2::new(12.0, 20.0),
                        cooldowns: CooldownState::<PlayerAbilities>::new([
                            (PlayerAbilities::BasicAttack, Cooldown::from_secs(0.8)),
                            (PlayerAbilities::SecondaryAttack, Cooldown::from_secs(2.0)),
                            (PlayerAbilities::Utility, Cooldown::from_secs(2.3)),
                            (PlayerAbilities::Ultimate, Cooldown::from_secs(15.0)),
                        ]),
                    },
                ),
            ]
            .into(),
        }
    }
}

/// Characters that can be chosen by players to play as
#[derive(Eq, PartialEq, Hash, Debug, EnumIter, Clone)]
pub(crate) enum CharacterType {
    Captain,
    Juggernaut,
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

/// Tag for indicating multiplayer association
#[derive(Component, Debug, Clone, PartialEq, AsRefStr)]
pub(crate) enum PlayerNum {
    One,
    Two,
    Three,
    Four,
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
#[derive(Resource, Default)]
pub(super) struct ChosenCharactersResource {
    pub players: Vec<(PlayerNum, CharacterType)>,
}

/// Event for transferring character choices from ui to ChosenCharactersResource
#[derive(Event)]
pub(crate) struct ChosenCharactersEvent {
    pub players: Vec<(PlayerNum, CharacterType)>,
}
