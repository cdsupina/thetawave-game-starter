use bevy::prelude::{Component, States};

/// States enum for managing the high-level application flow
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub(crate) enum AppState {
    /// Initial state when app launches, displayed when game first starts up
    #[default]
    MainMenuLoading,
    /// State for the main menu to go to other parts of the game
    MainMenu,
    /// State for loading the game assets
    GameLoading,
    /// State for actually playing the game
    Game,
}

/// States enum for managing gameplay states
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub(crate) enum MainMenuState {
    /// Not in any menu state
    #[default]
    None,
    /// Title (main) menu state
    Title,
    /// Options menu state
    Options,
    /// Chracter selection state
    CharacterSelection,
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

/// States enum for managing pause menu states
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub(crate) enum PauseMenuState {
    #[default]
    None,
    Main,
    Options,
}

/// Component marker for main menu cleanup
#[derive(Component)]
pub(crate) struct MainMenuCleanup;

/// Component marker for title menu cleanup
#[derive(Component)]
pub(crate) struct TitleMenuCleanup;

/// Component marker for options menu cleanup
#[derive(Component)]
pub(crate) struct OptionsMenuCleanup;

/// Component marker for character selection cleanup
#[derive(Component)]
pub(crate) struct CharacterSelectionCleanup;

/// Component marker for in-game cleanup
#[derive(Component)]
pub(crate) struct GameCleanup;

/// Component marker for pause cleanup
#[derive(Component)]
pub(crate) struct PauseCleanup;

#[derive(Component)]
pub(crate) struct PauseMainCleanup;

#[derive(Component)]
pub(crate) struct PauseOptionsCleanup;
