mod data;
mod data_loader;
mod logging;
mod plugin;
mod states;
mod tags;

pub use data::{
    with_bloom, ALLY_BASE_COLOR, CollisionDamage, ENEMY_BASE_COLOR, Faction, HealthComponent,
    ParticleRenderingEnabled, XHITARA_BLOOD_COLOR,
};
pub use data_loader::{load_with_extended, merge_toml_values};
#[cfg(feature = "debug")]
pub use logging::LoggingSettings;
pub use plugin::ThetawaveCorePlugin;
pub use states::{
    AppState, Cleanup, DebugState, GameState, MainMenuState, PauseMenuState, ThetawaveStatesPlugin,
    ToggleDebugStateEvent, ToggleGameStateEvent,
};
pub use tags::PlayerTag;
