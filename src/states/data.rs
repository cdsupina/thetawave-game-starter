use bevy::prelude::{Component, States};

/// States enum for managing the high-level application flow
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub(crate) enum AppState {
    /// Initial state when app launches, displayed when game first starts up
    #[default]
    MainMenuLoading,
    /// State for the main menu to go to other parts of the game
    MainMenu,
    /// State for options/settings menu
    OptionsMenu,
    /// State for character selection screen where player chooses their character
    CharacterSelectionMenu,
    /// Active gameplay state when player is in the main game
    InGame,
}

/// States enum for managing gameplay states
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub(crate) enum GameState {
    /// Normal gameplay state
    #[default]
    Playing,
    /// Paused gameplay state
    Paused,
}

/// Component marker for main menu cleanup
#[derive(Component)]
pub(crate) struct MainMenuCleanup;

/// Component marker for character selection cleanup
#[derive(Component)]
pub(crate) struct CharacterSelectionCleanup;

/// Component marker for in-game cleanup
#[derive(Component)]
pub(crate) struct InGameCleanup;
