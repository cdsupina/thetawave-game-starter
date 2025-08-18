use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs, common_conditions::on_event},
    state::condition::in_state,
};
use thetawave_states::{AppState, GameState};

use crate::{
    SpawnProjectileEvent, attributes::ThetawaveAttributesPlugin, spawn::spawn_projectile_system,
};

/// Plugin for projectile spawning systems and events.
pub struct ThetawaveProjectilesPlugin;

impl Plugin for ThetawaveProjectilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(ThetawaveAttributesPlugin)
            .add_systems(
                Update,
                spawn_projectile_system.run_if(
                    in_state(AppState::Game)
                        .and(in_state(GameState::Playing).and(on_event::<SpawnProjectileEvent>)),
                ),
            )
            .add_event::<SpawnProjectileEvent>();
    }
}
