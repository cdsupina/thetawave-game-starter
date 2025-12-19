use bevy::app::Plugin;

use crate::{
    MobDeathEvent,
    attributes::{JointsComponent, MobAttributesComponent, MobSpawnerComponent},
};

pub struct ThetawaveAttributesPlugin;

impl Plugin for ThetawaveAttributesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<(JointsComponent, MobSpawnerComponent, MobAttributesComponent)>();
        app.add_message::<MobDeathEvent>();
    }
}
