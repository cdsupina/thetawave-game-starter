use bevy::{ecs::component::Component, prelude::Entity, reflect::Reflect};
use leafwing_abilities::Abilitylike;
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
    EnumIter,
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub(crate) enum InputType {
    #[default]
    Keyboard,
    Gamepad(Entity),
}

#[derive(Component)]
pub(crate) struct DummyGamepad;
