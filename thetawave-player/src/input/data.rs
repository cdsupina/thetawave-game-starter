use bevy::{
    ecs::{component::Component, entity::Entity, message::Message},
    reflect::Reflect,
};
use leafwing_input_manager::Actionlike;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumIter};

/// Actions for player entities in the game state
#[derive(
    Actionlike,
    Clone,
    Debug,
    Eq,
    Hash,
    PartialEq,
    Reflect,
    Serialize,
    Deserialize,
    EnumIter,
    AsRefStr,
)]
pub enum PlayerAction {
    Up,
    Down,
    Left,
    Right,
    Pause,
}

/// Abilities for player entities
#[derive(
    Actionlike,
    Clone,
    Debug,
    Eq,
    Hash,
    PartialEq,
    Reflect,
    Serialize,
    Deserialize,
    AsRefStr,
    EnumIter,
)]
pub enum PlayerAbility {
    BasicAttack,
    SecondaryAttack,
    Utility,
    Ultimate,
}

/// Actions for selecting a character from a carousel
#[derive(Actionlike, Clone, Debug, Eq, Hash, PartialEq, Reflect, Serialize, Deserialize)]
pub enum CharacterCarouselAction {
    CycleLeft,
    CycleRight,
    Ready,
    Unready,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub enum InputType {
    #[default]
    Keyboard,
    Gamepad(Entity),
}

#[derive(Component)]
pub struct DummyGamepad;

/// Tag for indicating multiplayer association
#[derive(Component, Debug, Clone, PartialEq, Eq, AsRefStr, Hash, PartialOrd, Ord)]
pub enum PlayerNum {
    One,
    Two,
    Three,
    Four,
}

impl PlayerNum {
    pub fn next(&self) -> Option<Self> {
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

/// Event for when a player presses a join button on character selection screen
#[derive(Message, Debug)]
pub struct PlayerJoinEvent {
    pub player_num: PlayerNum,
    pub input: InputType,
}
