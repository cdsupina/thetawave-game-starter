use bevy::app::Plugin;
use thetawave_core::load_with_extended;

use crate::attributes::data::ProjectileAttributesResource;

pub(crate) struct ThetawaveAttributesPlugin;

impl Plugin for ThetawaveAttributesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(load_with_extended::<ProjectileAttributesResource>(
            include_bytes!("../../data/projectile_attributes.toml"),
            "projectile_attributes.toml",
        ));
    }
}
