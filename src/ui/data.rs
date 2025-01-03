use bevy::prelude::Component;

use crate::states::{AppState, GameState, MainMenuState, PauseMenuState};

#[derive(Component, Debug)]
pub(super) enum ButtonAction {
    EnterAppState(AppState),
    EnterMainMenuState(MainMenuState),
    EnterGameState(GameState),
    EnterPauseMenuState(PauseMenuState),
    Exit,
    ApplyOptions,
    OpenBlueskyWebsite,
    OpenGithubWebsite,
}

impl TryFrom<&String> for ButtonAction {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "enter_main_pause" => Ok(Self::EnterPauseMenuState(PauseMenuState::Main)),
            "enter_playing" => Ok(Self::EnterGameState(GameState::Playing)),
            "enter_main_menu_options" => Ok(Self::EnterMainMenuState(MainMenuState::Options)),
            "enter_pause_menu_options" => Ok(Self::EnterPauseMenuState(PauseMenuState::Options)),
            "enter_main_menu" => Ok(Self::EnterAppState(AppState::MainMenuLoading)),
            "enter_character_selection" => {
                Ok(Self::EnterMainMenuState(MainMenuState::CharacterSelection))
            }
            "enter_game" => Ok(Self::EnterAppState(AppState::GameLoading)),
            "exit" => Ok(Self::Exit),
            "apply_options" => Ok(Self::ApplyOptions),
            "enter_title" => Ok(Self::EnterMainMenuState(MainMenuState::Title)),
            "open_bluesky_website" => Ok(Self::OpenBlueskyWebsite),
            "open_github_website" => Ok(Self::OpenGithubWebsite),
            _ => Err("Invalid action".to_string()),
        }
    }
}

/// Loading bar tag component
#[derive(Component)]
pub(super) struct LoadingBar;
