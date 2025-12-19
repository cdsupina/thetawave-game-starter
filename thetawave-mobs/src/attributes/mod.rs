mod data;
mod plugin;

pub use data::{
    JointsComponent, MobAttributesComponent, MobDeathEvent, MobMarker, MobSpawnerComponent,
    ProjectileSpawnerComponent,
};
pub(crate) use plugin::ThetawaveAttributesPlugin;
