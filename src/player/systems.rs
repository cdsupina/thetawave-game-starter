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
use leafwing_input_manager::{
    prelude::{ActionState, InputMap},
    InputManagerBundle,
};

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
        Name::new("Player"),
    ));
}

pub(super) fn player_move_system(
    mut player_action_q: Query<(&ActionState<PlayerAction>, &mut LinearVelocity)>,
) {
    for (player_action, mut lin_vel) in player_action_q.iter_mut() {
        for action in player_action.get_pressed().iter() {
            match action {
                PlayerAction::Up => lin_vel.y += 2.0,
                PlayerAction::Down => lin_vel.y -= 2.0,
                PlayerAction::Left => lin_vel.x -= 2.0,
                PlayerAction::Right => lin_vel.x += 2.0,
            }
        }
    }
}
