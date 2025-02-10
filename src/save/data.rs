use bevy::ecs::system::Resource;
use serde::{Deserialize, Serialize};

/// Save file for persisting player game progress
#[derive(Default, Resource, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SaveRes {
    /// Number of total games they player has played
    pub run_count: u32,
    /// Number of times the player has entered the victory or game over ends
    pub win_count: u32,
    pub loss_count: u32,
}
