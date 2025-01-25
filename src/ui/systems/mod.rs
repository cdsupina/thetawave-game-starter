use super::data::{
    ButtonAction, CarouselSlotPosition, CharacterCarousel, LoadingBar, MenuButtonState,
    PlayerJoinEvent, PlayerReadyEvent, VisibleCarouselSlot,
};
use crate::{
    assets::{LoadingProgressEvent, UiAssets},
    audio::AudioEffectEvent,
    input::InputType,
    options::{ApplyOptionsEvent, OptionsRes},
    player::{ChosenCharactersResource, PlayerNum},
    states::{AppState, Cleanup, GameState, MainMenuState, PauseMenuState},
};
use bevy::{
    app::AppExit,
    input::ButtonInput,
    prelude::{
        Children, Entity, EventReader, EventWriter, Gamepad, GamepadButton, KeyCode, MouseButton,
        NextState, Query, Res, ResMut, With,
    },
};
use bevy_alt_ui_navigation_lite::{events::NavEvent, prelude::Focusable};
use bevy_aseprite_ultra::prelude::AseUiAnimation;
use log::{info, warn};

pub(super) mod character_selection;
pub(super) mod hui;
pub(super) mod loading;
pub(super) mod options;
pub(super) mod pause;
pub(super) mod title;

const GITHUB_URL: &str = "https://github.com/thetawavegame/thetawave";
const BLUESKY_URL: &str = "https://bsky.app/profile/carlo.metalmancy.tech";

/// System that handles the focus state of menu buttons
/// Updates the animation state of buttons when focus changes
/// Takes navigation events and queries for focusable entities and their animations
pub(super) fn menu_button_focus_system(
    mut nav_events: EventReader<NavEvent>,
    focusable_q: Query<(&Children, &MenuButtonState), With<Focusable>>,
    mut ase_q: Query<&mut AseUiAnimation>,
    mut audio_effect_events: EventWriter<AudioEffectEvent>,
) {
    for event in nav_events.read() {
        if let NavEvent::FocusChanged { to, from } = event {
            if to != from {
                // Handle newly focused button
                if let Ok((children, button_state)) = focusable_q.get(*to.first()) {
                    // Play pressed button effect
                    audio_effect_events.send(AudioEffectEvent::MenuButtonPressed);

                    // Update the button animation
                    for child in children.iter() {
                        if let Ok(mut ase_animation) = ase_q.get_mut(*child) {
                            if matches!(button_state, MenuButtonState::Ready) {
                                ase_animation.animation.play_loop("ready_pressed");
                            } else {
                                ase_animation.animation.play_loop("pressed");
                            }
                        }
                    }
                }

                // Handle previously focused button
                if let Ok((children, button_state)) = focusable_q.get(*from.first()) {
                    // Play released button effect
                    audio_effect_events.send(AudioEffectEvent::MenuButtonReleased);

                    // Update the button animation
                    for child in children.iter() {
                        if let Ok(mut ase_animation) = ase_q.get_mut(*child) {
                            if matches!(button_state, MenuButtonState::Ready) {
                                ase_animation.animation.play_loop("ready_released");
                            } else {
                                ase_animation.animation.play_loop("released");
                            }
                        }
                    }
                }
            }
        }
    }
}

/// This system reads and performs navigation events from bevy_alt_ui_navigation, handling each button action accordingly.
/// If a player one has been registered, will only activate button actions from player one
pub(super) fn menu_button_action_system(
    mut nav_events: EventReader<NavEvent>,
    focusable_q: Query<&ButtonAction, With<Focusable>>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_pause_state: ResMut<NextState<PauseMenuState>>,
    mut exit_events: EventWriter<AppExit>,
    mut apply_options_events: EventWriter<ApplyOptionsEvent>,
    mut effect_events: EventWriter<AudioEffectEvent>,
    mut player_join_events: EventWriter<PlayerJoinEvent>,
    mut player_ready_events: EventWriter<PlayerReadyEvent>,
    key_code_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    gamepads_q: Query<(Entity, &Gamepad)>,
    chosen_characters_res: Res<ChosenCharactersResource>,
) {
    for event in nav_events.read() {
        if let NavEvent::NoChanges { from, .. } = event {
            if let Ok(button_action) = focusable_q.get(*from.first()) {
                // Try to get the input from gamepad
                let gamepad_input = gamepads_q.iter().find_map(|(entity, gamepad)| {
                    if gamepad.just_pressed(GamepadButton::South) {
                        Some(InputType::Gamepad(entity))
                    } else {
                        None
                    }
                });

                // Store the possible input from keyboard or gamepad
                let maybe_input_type = if key_code_input.just_pressed(KeyCode::Enter)
                    || mouse_button_input.just_released(MouseButton::Left)
                {
                    Some(InputType::Keyboard)
                } else {
                    gamepad_input
                };

                // If a player one has been registered, ensure that the given input is from player one
                let mut valid_input = true;

                if let Some(player_one_data) = chosen_characters_res.players.get(&PlayerNum::One) {
                    if let Some(input_type) = maybe_input_type.clone() {
                        if input_type != player_one_data.input {
                            valid_input = false;
                        }
                    }
                }

                if valid_input {
                    match button_action {
                        ButtonAction::EnterAppState(app_state) => {
                            info!("here");
                            next_app_state.set(*app_state);
                            effect_events.send(AudioEffectEvent::MenuButtonConfirm);
                        }
                        ButtonAction::EnterMainMenuState(main_menu_state) => {
                            next_main_menu_state.set(*main_menu_state);
                            effect_events.send(AudioEffectEvent::MenuButtonConfirm);
                        }
                        ButtonAction::EnterGameState(game_state) => {
                            next_game_state.set(*game_state);
                            effect_events.send(AudioEffectEvent::MenuButtonConfirm);
                        }
                        ButtonAction::EnterPauseMenuState(pause_menu_state) => {
                            next_pause_state.set(*pause_menu_state);
                            effect_events.send(AudioEffectEvent::MenuButtonConfirm);
                        }
                        ButtonAction::Exit => {
                            exit_events.send(AppExit::Success);
                            effect_events.send(AudioEffectEvent::MenuButtonConfirm);
                        }
                        ButtonAction::ApplyOptions => {
                            apply_options_events.send(ApplyOptionsEvent);
                            effect_events.send(AudioEffectEvent::MenuButtonConfirm);
                        }
                        ButtonAction::OpenBlueskyWebsite => {
                            open_website(BLUESKY_URL);
                            effect_events.send(AudioEffectEvent::MenuButtonConfirm);
                        }
                        ButtonAction::OpenGithubWebsite => {
                            open_website(GITHUB_URL);
                            effect_events.send(AudioEffectEvent::MenuButtonConfirm);
                        }
                        ButtonAction::Join(player_num) => {
                            // if a valid input type is read, check if it has already been registered
                            if let Some(input_type) = maybe_input_type {
                                if !chosen_characters_res.contains_input(input_type.clone()) {
                                    player_join_events.send(PlayerJoinEvent {
                                        player_num: player_num.clone(),
                                        input: input_type.clone(),
                                    });
                                    effect_events.send(AudioEffectEvent::MenuButtonConfirm);
                                }
                            }
                        }
                        ButtonAction::Ready(player_num) => {
                            // Check that the given input and the input of PlayerOne match
                            // Prevents keyboard for readying for gamepad player one
                            if let Some(input_type) = maybe_input_type {
                                if let Some(data) =
                                    chosen_characters_res.players.get(&PlayerNum::One)
                                {
                                    if data.input == input_type {
                                        player_ready_events.send(PlayerReadyEvent {
                                            player_num: player_num.clone(),
                                            is_ready: true,
                                        });
                                        effect_events.send(AudioEffectEvent::MenuButtonConfirm);
                                    }
                                }
                            }
                        }
                        ButtonAction::UnReady(player_num) => {
                            // Check that the given input and the input of PlayerOne match
                            // Prevents keyboard for readying for gamepad player one
                            if let Some(input_type) = maybe_input_type {
                                if let Some(data) =
                                    chosen_characters_res.players.get(&PlayerNum::One)
                                {
                                    if data.input == input_type {
                                        player_ready_events.send(PlayerReadyEvent {
                                            player_num: player_num.clone(),
                                            is_ready: false,
                                        });
                                        effect_events.send(AudioEffectEvent::MenuButtonConfirm);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// This function handles the opening of certain websites.
// It opens the URL in a web browser.
fn open_website(url: &str) {
    if webbrowser::open(url).is_ok() {
        // If opening the URL was successful, it is logged as an information.
        info!("Opening webiste: {url}");
    } else {
        // If opening the URL has failed, it is logged as a warning.
        warn!("Failed to open website: {url}");
    }
}
