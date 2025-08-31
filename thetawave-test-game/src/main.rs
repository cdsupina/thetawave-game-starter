use bevy::{app::App, input::keyboard::KeyCode, prelude::*};
use thetawave_starter::ThetawaveStarterPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins((ThetawaveStarterPlugin {
        window_title: "Thetawave Test".to_string(),
        starting_resolution: (1280., 720.),
        show_debug_keycode: KeyCode::Backquote,
    },));

    app.run();
}
/*
fn test_extended_assets_system(extended_assets: Res<ExtendedGameAssets>) {
    if extended_assets.sprites.is_empty() {
        return;
    }

    for (key, handle) in &extended_assets.sprites {
        info!("Extended asset loaded: {} -> {:?}", key, handle);
    }
}
*/
