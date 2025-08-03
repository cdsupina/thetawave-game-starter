use avian2d::prelude::PhysicsLayer;
#[cfg(feature = "physics_debug")]
use bevy::ecs::resource::Resource;
use serde::Deserialize;

#[cfg(feature = "physics_debug")]
#[derive(Resource, Default)]
pub struct PhysicsDebugSettings {
    pub gizmos_enabled: bool,
    pub diagnostics_enabled: bool,
}

#[derive(PhysicsLayer, Default, Deserialize, Debug, Clone)]
pub enum ThetawavePhysicsLayer {
    #[default]
    Enemy,
    Player,
    Ally,
    Tentacle,
}
