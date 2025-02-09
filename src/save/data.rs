use bevy::ecs::system::Resource;
use serde::{Deserialize, Serialize};

#[derive(Default, Resource, Serialize, Deserialize, Clone)]
pub(crate) struct SaveRes {
    pub run_count: u32,
    pub win_count: u32,
    pub loss_count: u32,
}
