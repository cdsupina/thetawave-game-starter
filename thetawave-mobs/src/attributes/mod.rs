mod data;
mod plugin;

pub use data::MobType;
pub(crate) use data::{
    JointedMob, JointsComponent, MobAttributesComponent, MobAttributesResource, MobComponentBundle,
    MobDecorationType, MobSpawnerComponent, ProjectileSpawnerComponent,
};
pub(crate) use plugin::ThetawaveAttributesPlugin;
