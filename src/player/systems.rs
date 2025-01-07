use crate::{
    assets::GameAssets,
    input::PlayerAction,
    states::{AppState, Cleanup},
};
use avian2d::prelude::{Collider, LinearVelocity, MaxLinearSpeed, RigidBody};
use bevy::{
    core::Name,
    prelude::{Commands, KeyCode, Query, Res},
};
use bevy_aseprite_ultra::prelude::{Animation, AseSpriteAnimation};
use bevy_egui::egui::Vec2;
use leafwing_input_manager::{
    prelude::{ActionState, InputMap},
    InputManagerBundle,
};

use super::data::PlayerStatsComponent;

pub(super) fn spawn_players_system(mut cmds: Commands, assets: Res<GameAssets>) {
    let input_map = InputMap::new([
        (PlayerAction::Up, KeyCode::KeyW),
        (PlayerAction::Down, KeyCode::KeyS),
        (PlayerAction::Left, KeyCode::KeyA),
        (PlayerAction::Right, KeyCode::KeyD),
    ]);

    cmds.spawn((
        AseSpriteAnimation {
            animation: Animation::tag("idle"),
            aseprite: assets.captain_character_aseprite.clone(),
        },
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        Collider::rectangle(6.0, 12.0),
        RigidBody::Kinematic,
        MaxLinearSpeed(100.0),
        InputManagerBundle::with_map(input_map),
        PlayerStatsComponent {
            acceleration: 2.0,
            deceleration_factor: 0.972,
        },
        Name::new("Player"),
    ));
}

pub(super) fn player_move_system(
    mut player_action_q: Query<(
        &PlayerStatsComponent,
        &ActionState<PlayerAction>,
        &mut LinearVelocity,
    )>,
) {
    for (player_stats, player_action, mut lin_vel) in player_action_q.iter_mut() {
        let mut dir_vec = Vec2::ZERO;

        for action in player_action.get_pressed().iter() {
            match action {
                PlayerAction::Up => dir_vec.y += 1.0,
                PlayerAction::Down => dir_vec.y -= 1.0,
                PlayerAction::Left => dir_vec.x -= 1.0,
                PlayerAction::Right => dir_vec.x += 1.0,
            }
        }

        let dir_vec_norm = dir_vec.normalized();

        lin_vel.x += dir_vec_norm.x * player_stats.acceleration;
        lin_vel.y += dir_vec_norm.y * player_stats.acceleration;

        // Decelerate when there is no input on a particular axis
        if dir_vec.x == 0.0 {
            lin_vel.x *= player_stats.deceleration_factor;
        }
        if dir_vec.y == 0.0 {
            lin_vel.y *= player_stats.deceleration_factor;
        }
    }
}
