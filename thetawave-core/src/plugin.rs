use bevy::{app::Plugin, prelude::App};

use crate::{Faction, HealthComponent};

pub struct ThetawaveCorePlugin;

impl Plugin for ThetawaveCorePlugin {
    fn build(&self, app: &mut App) {
        // Register core components for reflection/inspection
        app.register_type::<HealthComponent>();
        app.register_type::<Faction>();
    }
}
