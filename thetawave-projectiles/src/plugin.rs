use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs, common_conditions::on_event},
    state::condition::in_state,
};
use thetawave_states::{AppState, GameState};

use crate::{
    SpawnProjectileEvent, attributes::ThetawaveAttributesPlugin, spawn::spawn_projectile_system,
    systems::timed_range_system,
};

pub struct ThetawaveProjectilesPlugin;

impl Plugin for ThetawaveProjectilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(ThetawaveAttributesPlugin)
            .add_systems(
                Update,
                (
                    timed_range_system,
                    spawn_projectile_system.run_if(on_event::<SpawnProjectileEvent>),
                )
                    .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
            )
            .add_event::<SpawnProjectileEvent>();
    }
}
