mod data;
mod plugin;

pub use data::MobMarker;
pub(crate) use data::{
    JointedMob, JointsComponent, MobAttributesComponent, MobAttributesResource, MobComponentBundle,
    MobSpawnerComponent, ProjectileSpawnerComponent,
};
pub(crate) use plugin::ThetawaveAttributesPlugin;
