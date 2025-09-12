use crate::options::OptionsRes;
use avian2d::prelude::{
    Collider, CollisionLayers, LayerMask, LockedAxes, MaxLinearSpeed, Restitution, RigidBody,
};
use bevy::{
    asset::Handle,
    prelude::{Commands, Name, Res},
    sprite::Sprite,
    utils::default,
};
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation, Aseprite};
use bevy_persistent::Persistent;
use leafwing_abilities::AbilitiesBundle;
use leafwing_input_manager::prelude::InputMap;
use thetawave_assets::{AssetError, AssetResolver, ExtendedGameAssets, GameAssets};
use thetawave_core::{AppState, Cleanup};
use thetawave_core::{HealthComponent, PlayerTag};
use thetawave_physics::ThetawavePhysicsLayer;
use thetawave_player::{
    CharacterType, CharactersResource, ChosenCharactersResource, InputType, PlayerAbility,
    PlayerStats,
};

/// Get the Aseprite handle from a given CharacterType using asset resolver
fn get_character_sprite(
    character_type: &CharacterType,
    extended_assets: &ExtendedGameAssets,
    game_assets: &GameAssets,
) -> Result<Handle<Aseprite>, AssetError> {
    let key = match character_type {
        CharacterType::Captain => "captain_character",
        CharacterType::Juggernaut => "juggernaut_character",
        CharacterType::Doomwing => "doomwing_character",
    };

    AssetResolver::get_game_sprite(key, extended_assets, game_assets)
}

/// Spawn a player controlled entity
pub(super) fn spawn_players_system(
    mut cmds: Commands,
    game_assets: Res<GameAssets>,
    extended_assets: Res<ExtendedGameAssets>,
    options_res: Res<Persistent<OptionsRes>>,
    characters_res: Res<CharactersResource>,
    chosen_characters_res: Res<ChosenCharactersResource>,
) -> bevy::ecs::error::Result {
    // Iterate through all of the chosen characters
    for (player_num, chosen_character_data) in chosen_characters_res.players.iter() {
        // Spawn a player using the CharacterData from the character type
        if let Some(character_data) = characters_res
            .characters
            .get(&chosen_character_data.character)
        {
            let mut entity_cmds = cmds.spawn((
                player_num.clone(),
                AseAnimation {
                    animation: Animation::tag("idle"),
                    aseprite: get_character_sprite(
                        &chosen_character_data.character,
                        &extended_assets,
                        &game_assets,
                    )?,
                },
                Sprite::default(),
                Cleanup::<AppState> {
                    states: vec![AppState::Game],
                },
                Collider::rectangle(
                    character_data.collider_dimensions.x,
                    character_data.collider_dimensions.y,
                ),
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                MaxLinearSpeed(character_data.max_speed),
                Restitution::new(character_data.restitution),
                CollisionLayers::new([ThetawavePhysicsLayer::Player], [LayerMask::ALL]),
                match chosen_character_data.input {
                    InputType::Keyboard => {
                        InputMap::new(options_res.player_keyboard_action_input_mappings.clone())
                            .insert_multiple(options_res.player_mouse_action_input_mappings.clone())
                            .to_owned()
                    }
                    InputType::Gamepad(entity) => {
                        InputMap::new(options_res.player_gamepad_action_input_mappings.clone())
                            .with_gamepad(entity)
                    }
                },
                match chosen_character_data.input {
                    InputType::Keyboard => {
                        InputMap::new(options_res.player_keyboard_abilities_input_mappings.clone())
                            .insert_multiple(
                                options_res.player_mouse_abilities_input_mappings.clone(),
                            )
                            .to_owned()
                    }
                    InputType::Gamepad(entity) => {
                        InputMap::new(options_res.player_gamepad_abilities_input_mappings.clone())
                            .with_gamepad(entity)
                    }
                },
                AbilitiesBundle::<PlayerAbility> {
                    cooldowns: character_data.cooldowns.clone(),
                    ..default()
                },
                PlayerStats {
                    acceleration: character_data.acceleration,
                    deceleration_factor: character_data.deceleration_factor,
                },
                Name::new("Player"),
            ));

            // new insert because of the bundle size limit
            entity_cmds.insert((PlayerTag, HealthComponent::new(character_data.health)));
        }
    }
    Ok(())
}
