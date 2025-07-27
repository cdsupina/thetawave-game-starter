#[cfg(feature = "physics_debug")]
use bevy::ecs::resource::Resource;

#[cfg(feature = "physics_debug")]
#[derive(Resource, Default)]
pub struct PhysicsDebugSettings {
    pub gizmos_enabled: bool,
    pub diagnostics_enabled: bool,
}
