mod data;
mod plugin;
mod systems;

pub use data::{
    AppState, Cleanup, DebugState, GameState, MainMenuState, PauseMenuState, ToggleDebugStateEvent,
    ToggleGameStateEvent,
};
pub use plugin::ThetawaveStatesPlugin;
pub use systems::{enter_playing_state_system, enter_title_menu_state_system};
