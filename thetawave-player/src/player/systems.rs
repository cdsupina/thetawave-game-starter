use avian2d::prelude::LinearVelocity;
use bevy::{
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        system::Query,
    },
    log::{info, warn},
    math::Vec2,
};
use leafwing_abilities::prelude::CooldownState;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    EquippedAbilities, ExecutePlayerAbilityEvent, PlayerAbility, PlayerAction, PlayerDeathEvent,
    PlayerStats,
};

/// Move the player around by modifying their linear velocity
pub(crate) fn player_move_system(
    mut player_action_q: Query<(
        &PlayerStats,
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
                PlayerAction::Pause => {}
            }
        }

        // Normalize the direction vector
        let dir_vec_norm = dir_vec.normalize_or_zero();

        // Add the components of the direction vector to the x and y velocity components
        lin_vel.x += dir_vec_norm.x * player_stats.acceleration;
        lin_vel.y += dir_vec_norm.y * player_stats.acceleration;

        // Decelerate when there is no input on a particular axis
        if dir_vec.x == 0.0 {
            lin_vel.x *= player_stats.deceleration;
        }
        if dir_vec.y == 0.0 {
            lin_vel.y *= player_stats.deceleration;
        }

        // If speed exceeds max_speed, apply deceleration to bring it back down
        let current_speed = lin_vel.length();
        if current_speed > player_stats.max_speed {
            lin_vel.x *= player_stats.deceleration;
            lin_vel.y *= player_stats.deceleration;
        }
    }
}

/// System for activating player abilities when ready
pub(crate) fn player_ability_system(
    mut player_ability_q: Query<(
        Entity,
        &mut CooldownState<PlayerAbility>,
        &EquippedAbilities,
        &ActionState<PlayerAbility>,
    )>,
    mut execute_ability_event_writer: EventWriter<ExecutePlayerAbilityEvent>,
) {
    for (entity, mut cooldown_state, equipped_abilities, action_state) in
        player_ability_q.iter_mut()
    {
        for ability in action_state.get_pressed() {
            if cooldown_state.trigger(&ability).is_ok() {
                if let Some(ability_type) = equipped_abilities.abilities.get(&ability) {
                    let execute_ability_event = ExecutePlayerAbilityEvent {
                        player_entity: entity,
                        ability_type: ability_type.clone(),
                    };
                    info!("{:?}: {:?}", ability, execute_ability_event);
                    execute_ability_event_writer.write(execute_ability_event);
                } else {
                    warn!(
                        "Player attempted to use ability {:?} but it's not equipped",
                        ability
                    );
                }
            }
        }
    }
}

pub(crate) fn player_death_system(mut player_death_event_reader: EventReader<PlayerDeathEvent>) {
    for event in player_death_event_reader.read() {
        info!("Player {} died.", event.player_entity);
    }
}
