use avian2d::prelude::CollisionStarted;
use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs, common_conditions::on_event},
    state::condition::in_state,
};
use bevy_aseprite_ultra::prelude::AnimationEvents;
use thetawave_states::{AppState, GameState};

use crate::{
    SpawnProjectileEvent,
    attributes::{SpawnProjectileEffectEvent, ThetawaveAttributesPlugin},
    spawn::{spawn_effect_system, spawn_projectile_system},
    systems::{despawn_after_animation_system, projectile_hit_system, timed_range_system},
};

/// Plugin for projectile spawning systems and events.
pub struct ThetawaveProjectilesPlugin;

impl Plugin for ThetawaveProjectilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(ThetawaveAttributesPlugin)
            .add_systems(
                Update,
                (
                    timed_range_system,
                    projectile_hit_system.run_if(on_event::<CollisionStarted>),
                    spawn_projectile_system.run_if(on_event::<SpawnProjectileEvent>),
                    spawn_effect_system.run_if(on_event::<SpawnProjectileEffectEvent>),
                    despawn_after_animation_system.run_if(on_event::<AnimationEvents>),
                )
                    .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
            )
            .add_event::<SpawnProjectileEvent>()
            .add_event::<SpawnProjectileEffectEvent>();
    }
}
