use bevy::app::Plugin;
use toml::from_slice;

use crate::attributes::{
    JointsComponent, MobAttributesComponent, MobSpawnerComponent, data::MobAttributesResource,
};

pub struct ThetawaveAttributesPlugin;

impl Plugin for ThetawaveAttributesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<(JointsComponent, MobSpawnerComponent, MobAttributesComponent)>();
        app.insert_resource(
            from_slice::<MobAttributesResource>(include_bytes!(
                "../../../assets/data/mob_attributes.toml"
            ))
            .expect("Failed to parse MobAttributesResource from `mob_attributes.toml`."),
        );
    }
}
