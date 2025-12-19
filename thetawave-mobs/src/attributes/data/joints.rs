use bevy::{
    ecs::{component::Component, entity::Entity},
    platform::collections::HashMap,
    reflect::Reflect,
};

/// Hashmap of joints connected to a mob
/// This is for "anchors" only
/// Used by behaviors for referencing joint entities
#[derive(Component, Reflect)]
pub struct JointsComponent {
    pub joints: HashMap<String, Entity>,
}
