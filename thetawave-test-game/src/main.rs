use bevy::{
    app::App,
    input::keyboard::KeyCode,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use thetawave_starter::ThetawaveStarterPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins((ThetawaveStarterPlugin {
        window_title: "Thetawave Test".to_string(),
        starting_resolution: (1280., 720.),
        show_debug_keycode: KeyCode::Backquote,
        extended_abilities: HashMap::new(),
        extended_duration_abilities: HashSet::new(),
    },));

    app.run();
}
