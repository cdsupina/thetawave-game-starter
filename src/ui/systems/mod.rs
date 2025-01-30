use super::data::{
    ButtonAction, ButtonActionDelayTimer, CarouselSlotPosition, CharacterCarousel,
    DelayedButtonPressEvent, LoadingBar, MenuButtonState, PlayerJoinEvent, PlayerReadyEvent,
    VisibleCarouselSlot,
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
    core::Name,
    hierarchy::{BuildChildren, ChildBuild, ChildBuilder},
    input::ButtonInput,
    prelude::{
        Children, Entity, EventReader, EventWriter, Gamepad, GamepadButton, KeyCode, Local,
        MouseButton, NextState, Query, Res, ResMut, With,
    },
    text::TextFont,
    time::Time,
    ui::{widget::Text, FlexDirection, JustifyContent, Node, UiRect, Val},
    utils::default,
};
use bevy_alt_ui_navigation_lite::{events::NavEvent, prelude::Focusable};
use bevy_aseprite_ultra::prelude::{Animation, AseUiAnimation};
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
                    audio_effect_events.send(AudioEffectEvent::MenuButtonSelected);

                    // Update the button animation
                    for child in children.iter() {
                        if let Ok(mut ase_animation) = ase_q.get_mut(*child) {
                            if matches!(button_state, MenuButtonState::Ready) {
                                ase_animation.animation.play_loop("ready_selected");
                            } else {
                                ase_animation.animation.play_loop("selected");
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
    focusable_q: Query<(Entity, &ButtonAction), With<Focusable>>,
    mut effect_events: EventWriter<AudioEffectEvent>,
    key_code_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    gamepads_q: Query<(Entity, &Gamepad)>,
    chosen_characters_res: Res<ChosenCharactersResource>,
    mut button_action_events: EventWriter<DelayedButtonPressEvent>,
    mut player_join_events: EventWriter<PlayerJoinEvent>,
    mut player_ready_events: EventWriter<PlayerReadyEvent>,
) {
    for event in nav_events.read() {
        if let NavEvent::NoChanges { from, .. } = event {
            if let Ok((button_entity, button_action)) = focusable_q.get(*from.first()) {
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

                if let Some(input_type) = maybe_input_type {
                    let mut valid_input = true;

                    // If a player one has been registered, ensure that the given input is from player one
                    if let Some(player_one_data) =
                        chosen_characters_res.players.get(&PlayerNum::One)
                    {
                        if input_type != player_one_data.input {
                            valid_input = false;
                        }
                    }

                    if valid_input {
                        // Activate join and ready actions instantly, delay other actions by sending DelayedButtonPressEvent
                        match button_action {
                            ButtonAction::Join(player_num) => {
                                player_join_events.send(PlayerJoinEvent {
                                    player_num: player_num.clone(),
                                    input: input_type,
                                });
                            }
                            ButtonAction::Ready(player_num) => {
                                player_ready_events.send(PlayerReadyEvent {
                                    player_num: player_num.clone(),
                                    is_ready: true,
                                });
                            }
                            ButtonAction::UnReady(player_num) => {
                                player_ready_events.send(PlayerReadyEvent {
                                    player_num: player_num.clone(),
                                    is_ready: false,
                                });
                            }
                            _ => {
                                button_action_events.send(DelayedButtonPressEvent {
                                    button_action: button_action.clone(),
                                    button_entity,
                                });
                            }
                        }

                        // Play the button confirm sound
                        effect_events.send(AudioEffectEvent::MenuButtonConfirm);
                    }
                }
            }
        }
    }
}

pub(super) fn menu_button_delayed_action_system(
    mut button_press_events: EventReader<DelayedButtonPressEvent>,
    mut button_action_timer: Local<ButtonActionDelayTimer>,
    mut queued_button_action: Local<Option<ButtonAction>>,
    time: Res<Time>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_pause_state: ResMut<NextState<PauseMenuState>>,
    mut exit_events: EventWriter<AppExit>,
    mut apply_options_events: EventWriter<ApplyOptionsEvent>,
    focusable_q: Query<&Children, With<Focusable>>,
    mut ase_q: Query<&mut AseUiAnimation>,
) {
    // Check if there is a queued button action
    if let Some(button_action) = queued_button_action.clone() {
        // Tick the delay timer
        button_action_timer.0.tick(time.delta());

        // When finished clear the queue and execute the action
        if button_action_timer.0.just_finished() {
            *queued_button_action = None;
            match button_action {
                ButtonAction::EnterAppState(app_state) => {
                    next_app_state.set(app_state);
                }
                ButtonAction::EnterMainMenuState(main_menu_state) => {
                    next_main_menu_state.set(main_menu_state);
                }
                ButtonAction::EnterGameState(game_state) => {
                    next_game_state.set(game_state);
                }
                ButtonAction::EnterPauseMenuState(pause_menu_state) => {
                    next_pause_state.set(pause_menu_state);
                }
                ButtonAction::Exit => {
                    exit_events.send(AppExit::Success);
                }
                ButtonAction::ApplyOptions => {
                    apply_options_events.send(ApplyOptionsEvent);
                }
                ButtonAction::OpenBlueskyWebsite => {
                    open_website(BLUESKY_URL);
                }
                ButtonAction::OpenGithubWebsite => {
                    open_website(GITHUB_URL);
                }
                _ => {}
            }
        }
    } else if let Some(event) = button_press_events.read().next() {
        // Queue a button action if an event was sent while the queue is empty
        *queued_button_action = Some(event.button_action.clone());

        // Change the sprite of the button
        if let Ok(children) = focusable_q.get(event.button_entity) {
            for child in children.iter() {
                if let Ok(mut ase_animation) = ase_q.get_mut(*child) {
                    ase_animation.animation.play_loop("pressed");
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

trait ChildBuilderExt {
    fn spawn_menu_button(
        &mut self,
        ui_assets: &UiAssets,
        button_action: ButtonAction,
        width: f32,
        is_first: bool,
    );
}

impl ChildBuilderExt for ChildBuilder<'_> {
    /// Spawns a rectangular stylized menu button
    fn spawn_menu_button(
        &mut self,
        ui_assets: &UiAssets,
        button_action: ButtonAction,
        width: f32,
        is_first: bool,
    ) {
        self.spawn((
            Name::new("Menu Button"),
            Node {
                margin: UiRect::all(Val::Vh(1.0)),
                ..default()
            },
            button_action.clone(),
            MenuButtonState::Normal,
            Focusable::default(),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Menu Button Sprite"),
                    Node {
                        width: Val::Px(width),
                        aspect_ratio: Some(162.0 / 39.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    AseUiAnimation {
                        animation: Animation::tag(if is_first { "selected" } else { "released" }),
                        aseprite: ui_assets.menu_button_aseprite.clone(),
                    },
                ))
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            margin: UiRect::bottom(Val::Px(14.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::FlexEnd,
                            ..default()
                        })
                        .with_children(|parent| {
                            if let Some(button_text) = button_action.to_string() {
                                parent.spawn((
                                    Text::new(button_text),
                                    TextFont::from_font_size(25.0),
                                ));
                            }
                        });
                });
        });
    }
}
