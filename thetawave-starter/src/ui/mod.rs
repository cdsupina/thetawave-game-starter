mod data;
mod plugin;
mod systems;

pub(crate) use data::{GameEndResultResource, GameEndResultType, PlayerJoinEvent};
pub(crate) use plugin::ThetawaveUiPlugin;
pub use systems::update_egui_scale_system;
