use crate::{
    assets::GameAssets,
    input::PlayerAction,
    options::OptionsRes,
    states::{AppState, Cleanup},
};
use avian2d::prelude::{Collider, LinearVelocity, MaxLinearSpeed, RigidBody};
use bevy::{
    core::Name,
    prelude::{Commands, Query, Res},
};
use bevy_aseprite_ultra::prelude::{Animation, AseSpriteAnimation};
use bevy_egui::egui::Vec2;
use bevy_persistent::Persistent;
use leafwing_input_manager::{prelude::ActionState, InputManagerBundle};

use super::data::{CharactersResource, PlayerStatsComponent};

/// Spawn a player controlled entity
pub(super) fn spawn_players_system(
    mut cmds: Commands,
    assets: Res<GameAssets>,
    options_res: Res<Persistent<OptionsRes>>,
    characters_res: Res<CharactersResource>,
) {
    // Get the captain character for now until character selection is added
    if let Some(character) = characters_res.characters.get("captain") {
        cmds.spawn((
            AseSpriteAnimation {
                animation: Animation::tag("idle"),
                aseprite: assets.captain_character_aseprite.clone(),
            },
            Cleanup::<AppState> {
                states: vec![AppState::Game],
            },
            Collider::rectangle(
                character.collider_dimensions.x,
                character.collider_dimensions.y,
            ),
            RigidBody::Kinematic,
            MaxLinearSpeed(character.max_speed),
            InputManagerBundle::with_map(options_res.player_input_map.clone()),
            PlayerStatsComponent {
                acceleration: character.acceleration,
                deceleration_factor: character.deceleration_factor,
            },
            Name::new("Player"),
        ));
    }
}

/// Move the player around by modifying their linear velocity
pub(super) fn player_move_system(
    mut player_action_q: Query<(
        &PlayerStatsComponent,
        &ActionState<PlayerAction>,
        &mut LinearVelocity,
    )>,
) {
    for (player_stats, player_action, mut lin_vel) in player_action_q.iter_mut() {
        // Create a direction vector using the player's inputs
        let mut dir_vec = Vec2::ZERO;

        for action in player_action.get_pressed().iter() {
            match action {
                PlayerAction::Up => dir_vec.y += 1.0,
                PlayerAction::Down => dir_vec.y -= 1.0,
                PlayerAction::Left => dir_vec.x -= 1.0,
                PlayerAction::Right => dir_vec.x += 1.0,
            }
        }

        // Normalize the direction vector
        let dir_vec_norm = dir_vec.normalized();

        // Add the components of the direction vector to the x and y velocity components
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
