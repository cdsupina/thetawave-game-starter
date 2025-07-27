use bevy::ecs::resource::Resource;

#[derive(Resource, Default)]
pub struct PhysicsDebugSettings {
    pub gizmos_enabled: bool,
    pub diagnostics_enabled: bool,
}
