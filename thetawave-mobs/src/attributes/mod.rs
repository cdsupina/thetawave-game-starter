mod data;
mod plugin;

pub use data::MobType;
pub(crate) use data::{
    JointedMob, JointsComponent, MobAttributesComponent, MobAttributesResource, MobDecorationType,
    MobSpawnerComponent, ProjectileSpawnerComponent,
};
pub(crate) use plugin::ThetawaveAttributesPlugin;
