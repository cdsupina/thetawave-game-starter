use bevy::{app::App, input::keyboard::KeyCode};
use thetawave_starter::ThetawaveStarterPlugin;

use crate::ui::ThetawaveInfiniteUiPlugin;

mod ui;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        ThetawaveStarterPlugin {
            window_title: "Thetawave Infinite".to_string(),
            starting_resolution: (1280., 720.),
            show_debug_keycode: KeyCode::Backquote,
        },
        ThetawaveInfiniteUiPlugin,
    ));
    app.run();
}
