use bevy::app::Plugin;
use thetawave_core::load_with_extended;

use crate::{
    MobDeathEvent,
    attributes::{
        JointsComponent, MobAttributesComponent, MobSpawnerComponent, data::MobAttributesResource,
    },
};

pub struct ThetawaveAttributesPlugin;

impl Plugin for ThetawaveAttributesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<(JointsComponent, MobSpawnerComponent, MobAttributesComponent)>();
        app.add_event::<MobDeathEvent>();
        app.insert_resource(load_with_extended::<MobAttributesResource>(
            include_bytes!("../../data/mob_attributes.toml"),
            "mob_attributes.toml",
        ));
    }
}
