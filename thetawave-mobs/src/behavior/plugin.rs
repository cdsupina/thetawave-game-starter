use bevy::{
    app::{Plugin, Update},
    ecs::{
        schedule::{Condition, IntoScheduleConfigs},
        system::Res,
    },
    state::condition::in_state,
};
use bevy_behave::prelude::BehavePlugin;
use thetawave_states::{AppState, GameState};

use crate::behavior::{
    BehaviorReceiverComponent, MobBehaviorsResource,
    data::{TargetComponent, TransmitBehaviorEvent},
    systems::{
        brake_angular_system, brake_horizontal_system, directional_movement_system,
        do_for_time_system, find_player_target_system, lose_target_system, move_forward_system,
        move_to_system, move_to_target_system, receive_system, rotate_to_target_system,
        spawn_mob_system, spawn_projectile_system, transmit_system,
    },
};

pub(crate) struct ThetawaveMobBehaviorPlugin;

impl Plugin for ThetawaveMobBehaviorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BehavePlugin::default());
        app.add_event::<TransmitBehaviorEvent>();
        app.insert_resource(MobBehaviorsResource::new());

        // Register types for access in the inspector
        app.register_type::<(BehaviorReceiverComponent, TargetComponent)>();
        app.add_systems(
            Update,
            (
                directional_movement_system, // Handles MoveDown, MoveUp, MoveLeft, MoveRight
                brake_horizontal_system,
                move_to_system,
                find_player_target_system,
                move_to_target_system,
                rotate_to_target_system,
                move_forward_system,
                lose_target_system,
                brake_angular_system,
                spawn_mob_system,
                do_for_time_system,
                transmit_system,
                receive_system,
                spawn_projectile_system,
            )
                .run_if({
                    #[cfg(feature = "debug")]
                    {
                        use crate::MobDebugSettings;
                        in_state(AppState::Game)
                            .and(in_state(GameState::Playing))
                            .and(|mob_res: Res<MobDebugSettings>| mob_res.behaviors_enabled)
                    }
                    #[cfg(not(feature = "debug"))]
                    {
                        in_state(AppState::Game).and(in_state(GameState::Playing))
                    }
                }),
        );
    }
}
