use bevy::{prelude::Entity, reflect::Reflect};
use leafwing_abilities::Abilitylike;
use leafwing_input_manager::Actionlike;
use serde::{Deserialize, Serialize};
use strum::AsRefStr;

/// Actions for player entities in the game state
#[derive(Actionlike, Clone, Debug, Eq, Hash, PartialEq, Reflect, Serialize, Deserialize)]
pub(crate) enum PlayerAction {
    Up,
    Down,
    Left,
    Right,
    Pause,
}

/// Abilities for player entities
#[derive(
    Actionlike,
    Abilitylike,
    Clone,
    Debug,
    Eq,
    Hash,
    PartialEq,
    Reflect,
    Serialize,
    Deserialize,
    AsRefStr,
    Copy,
)]
pub(crate) enum PlayerAbility {
    BasicAttack,
    SecondaryAttack,
    Utility,
    Ultimate,
}

/// Actions for selecting a character from a carousel
#[derive(Actionlike, Clone, Debug, Eq, Hash, PartialEq, Reflect, Serialize, Deserialize)]
pub(crate) enum CharacterCarouselAction {
    CycleLeft,
    CycleRight,
    Ready,
    Unready,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum InputType {
    Keyboard,
    Gamepad(Entity),
}
