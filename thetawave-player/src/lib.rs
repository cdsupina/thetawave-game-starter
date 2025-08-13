use bevy::prelude::Component;
use serde::Deserialize;
use strum_macros::EnumIter;

/// Component for storing values used in systems for player entities
#[derive(Component)]
pub struct PlayerStats {
    pub acceleration: f32,
    pub deceleration_factor: f32,
}

/// Characters that can be chosen by players to play as
#[derive(Eq, PartialEq, Hash, Debug, EnumIter, Clone, Deserialize)]
pub enum CharacterType {
    Captain,
    Juggernaut,
    Doomwing,
}
