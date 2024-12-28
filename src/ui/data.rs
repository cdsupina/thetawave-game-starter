use bevy::prelude::Component;

#[derive(Component, Debug)]
pub(super) enum ButtonAction {
    EnterOptions,
    EnterCharacterSelection,
    Exit,
    ApplyOptions,
    EnterTitle,
    OpenBlueskyWebsite,
    OpenGithubWebsite,
}

impl TryFrom<&String> for ButtonAction {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "enter_options" => Ok(Self::EnterOptions),
            "enter_character_selection" => Ok(Self::EnterCharacterSelection),
            "exit" => Ok(Self::Exit),
            "apply_options" => Ok(Self::ApplyOptions),
            "enter_title" => Ok(Self::EnterTitle),
            "open_bluesky_website" => Ok(Self::OpenBlueskyWebsite),
            "open_github_website" => Ok(Self::OpenGithubWebsite),
            _ => Err("Invalid action".to_string()),
        }
    }
}
