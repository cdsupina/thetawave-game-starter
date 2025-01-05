use bevy::app::Plugin;
use leafwing_input_manager::plugin::InputManagerPlugin;

use super::data::PlayerAction;

/// Plugin for managing player entities
pub(crate) struct ThetawavePlayerPlugin;

impl Plugin for ThetawavePlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
    }
}
