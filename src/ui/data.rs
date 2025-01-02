use bevy::prelude::Component;

#[derive(Component, Debug)]
pub(super) enum ButtonAction {
    EnterMainPause,
    EnterPlaying,
    EnterMainMenuOptions,
    EnterPauseMenuOptions,
    EnterCharacterSelection,
    EnterGame,
    Exit,
    ApplyOptions,
    EnterTitle, // should be used for switching between MainMenuStates
    OpenBlueskyWebsite,
    OpenGithubWebsite,
    EnterMainMenu, // should be used for switching between AppStates
}

impl TryFrom<&String> for ButtonAction {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "enter_main_pause" => Ok(Self::EnterMainPause),
            "enter_playing" => Ok(Self::EnterPlaying),
            "enter_main_menu_options" => Ok(Self::EnterMainMenuOptions),
            "enter_pause_menu_options" => Ok(Self::EnterPauseMenuOptions),
            "enter_main_menu" => Ok(Self::EnterMainMenu),
            "enter_character_selection" => Ok(Self::EnterCharacterSelection),
            "enter_game" => Ok(Self::EnterGame),
            "exit" => Ok(Self::Exit),
            "apply_options" => Ok(Self::ApplyOptions),
            "enter_title" => Ok(Self::EnterTitle),
            "open_bluesky_website" => Ok(Self::OpenBlueskyWebsite),
            "open_github_website" => Ok(Self::OpenGithubWebsite),
            _ => Err("Invalid action".to_string()),
        }
    }
}
