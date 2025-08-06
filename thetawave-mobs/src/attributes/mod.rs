mod data;
mod plugin;

pub(crate) use data::{
    JointedMob, MobAttributesComponent, MobAttributesResource, MobDecorationType,
    MobSpawnerComponent,
};
pub use data::{MobType, SpawnMobEvent};
pub(crate) use plugin::ThetawaveAttributesPlugin;
