use bevy::{app::Plugin, prelude::App};

use crate::{Faction, HealthComponent, ParticleRenderingEnabled};
#[cfg(feature = "debug")]
use crate::LoggingSettings;

pub struct ThetawaveCorePlugin;

impl Plugin for ThetawaveCorePlugin {
    fn build(&self, app: &mut App) {
        // Register core components for reflection/inspection
        app.register_type::<HealthComponent>();
        app.register_type::<Faction>();

        // Initialize particle rendering control
        app.init_resource::<ParticleRenderingEnabled>();

        // Initialize logging settings resource (debug builds only)
        #[cfg(feature = "debug")]
        app.init_resource::<LoggingSettings>();
    }
}
