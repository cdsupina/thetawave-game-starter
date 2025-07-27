mod data;
mod plugin;
mod systems;

#[cfg(feature = "physics_debug")]
pub use data::PhysicsDebugSettings;

pub use plugin::ThetawavePhysicsPlugin;
