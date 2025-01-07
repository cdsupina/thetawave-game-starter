use bevy::reflect::Reflect;
use leafwing_abilities::Abilitylike;
use leafwing_input_manager::Actionlike;
use serde::{Deserialize, Serialize};

/// Actions for player entities in the game state
#[derive(Actionlike, Clone, Debug, Eq, Hash, PartialEq, Reflect, Serialize, Deserialize)]
pub(crate) enum PlayerAction {
    Up,
    Down,
    Left,
    Right,
}

/// Abilities for player entities
#[derive(
    Actionlike, Abilitylike, Clone, Debug, Eq, Hash, PartialEq, Reflect, Serialize, Deserialize,
)]
pub(crate) enum PlayerAbilities {
    BasicAttack,
    SecondaryAttack,
    Utility,
    Ultimate,
}
