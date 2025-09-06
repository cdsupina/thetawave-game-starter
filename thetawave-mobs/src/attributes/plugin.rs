use bevy::app::Plugin;
use thetawave_assets::load_with_extended;

use crate::attributes::{
    JointsComponent, MobAttributesComponent, MobSpawnerComponent, data::MobAttributesResource,
};

pub struct ThetawaveAttributesPlugin;

impl Plugin for ThetawaveAttributesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<(JointsComponent, MobSpawnerComponent, MobAttributesComponent)>();
        app.insert_resource(
            load_with_extended::<MobAttributesResource>(
                include_bytes!("../../../assets/data/mob_attributes.toml"),
                "mob_attributes.toml"
            )
        );
    }
}
