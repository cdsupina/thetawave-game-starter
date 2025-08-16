use bevy::app::Plugin;
use toml::from_slice;

use crate::attributes::data::ProjectileAttributesResource;

pub(crate) struct ThetawaveAttributesPlugin;

impl Plugin for ThetawaveAttributesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(
            from_slice::<ProjectileAttributesResource>(include_bytes!(
                "../../../assets/data/projectile_attributes.toml"
            ))
            .expect(
                "Failed to parse ProjectileAttributesResource from `projectile_attributes.toml`.",
            ),
        );
    }
}
