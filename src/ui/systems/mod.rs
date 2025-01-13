use super::data::{
    ButtonAction, CarouselSlotPosition, CharacterCarousel, LoadingBar, VisibleCarouselSlot,
};
use crate::{
    assets::{LoadingProgressEvent, UiAssets},
    audio::AudioEffectEvent,
    options::{ApplyOptionsEvent, OptionsRes},
    player::PlayerNum,
    states::{AppState, Cleanup, GameState, MainMenuState, PauseMenuState},
};
use bevy::{
    app::AppExit,
    prelude::{Children, EventReader, EventWriter, NextState, Query, ResMut, With},
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
    focusable_q: Query<&Children, With<Focusable>>,
    mut ase_q: Query<&mut AseUiAnimation>,
    mut audio_effect_events: EventWriter<AudioEffectEvent>,
) {
    for event in nav_events.read() {
        if let NavEvent::FocusChanged { to, from } = event {
            if to != from {
                // Handle newly focused button
                if let Ok(children) = focusable_q.get(*to.first()) {
                    // Play pressed button effect
                    audio_effect_events.send(AudioEffectEvent::MenuButtonPressed);

                    // Update the button animation
                    for child in children.iter() {
                        if let Ok(mut ase_animation) = ase_q.get_mut(*child) {
                            ase_animation.animation.play_loop("pressed");
                        }
                    }
                }

                // Handle previously focused button
                if let Ok(children) = focusable_q.get(*from.first()) {
                    // Play released button effect
                    audio_effect_events.send(AudioEffectEvent::MenuButtonReleased);

                    // Update the button animation
                    for child in children.iter() {
                        if let Ok(mut ase_animation) = ase_q.get_mut(*child) {
                            ase_animation.animation.play_loop("released");
                        }
                    }
                }
            }
        }
    }
}

/// This system reads and performs navigation events from bevy_alt_ui_navigation, handling each button action accordingly.
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
) {
    for event in nav_events.read() {
        if let NavEvent::NoChanges { from, .. } = event {
            if let Ok(button_action) = focusable_q.get(*from.first()) {
                effect_events.send(AudioEffectEvent::MenuButtonConfirm);

                match button_action {
                    ButtonAction::EnterAppState(app_state) => {
                        next_app_state.set(*app_state);
                    }
                    ButtonAction::EnterMainMenuState(main_menu_state) => {
                        next_main_menu_state.set(*main_menu_state);
                    }
                    ButtonAction::EnterGameState(game_state) => {
                        next_game_state.set(*game_state);
                    }
                    ButtonAction::EnterPauseMenuState(pause_menu_state) => {
                        next_pause_state.set(*pause_menu_state);
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
                    ButtonAction::ChracterCycleLeft(_player_num) => {
                        todo!("Cycle the character selection carousel left for player.");
                    }
                    ButtonAction::CharacterCycleRight(_player_num) => {
                        todo!("Cycle the character selection carousel right for player.");
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
