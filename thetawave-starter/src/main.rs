use bevy::{
    app::App,
    prelude::{DefaultPlugins, ImagePlugin, PluginGroup},
    utils::default,
    window::{Window, WindowMode, WindowPlugin, WindowResolution},
};
use bevy_aseprite_ultra::AsepriteUltraPlugin;
use thetawave_states::ThetawaveStatesPlugin;

mod assets;
mod audio;
mod backgrounds;
mod camera;
mod core;
mod input;
mod options;
mod physics;
mod player;
mod save;
mod ui;
mod window;

const PRIMARY_WINDOW_TITLE: &str = "Thetawave Starter Template";
const STARTING_WINDOW_RESOLUTION: (f32, f32) = (1280.0, 720.0);

fn main() {
    let mut app = App::new();

    app.add_plugins((
        assets::ThetawaveAssetsPlugin, // must be registered before AssetPlugin due to EmbeddedAssetPlugin
        DefaultPlugins
            .set(ImagePlugin::default_nearest()) // necessary for crisp pixel art
            .set(WindowPlugin {
                primary_window: Some(Window {
                    mode: WindowMode::Windowed,
                    resolution: WindowResolution::from(STARTING_WINDOW_RESOLUTION)
                        .with_scale_factor_override(1.0),
                    resizable: false,
                    title: PRIMARY_WINDOW_TITLE.to_string(),
                    ..default()
                }),
                ..default()
            }),
        AsepriteUltraPlugin, // plugin for using Aseprite assets
        // custom plugins for Thetawave
        ui::ThetawaveUiPlugin,
        options::ThetawaveOptionsPlugin,
        input::ThetawaveInputPlugin,
        ThetawaveStatesPlugin,
        camera::ThetawaveCameraPlugin,
        backgrounds::ThetawaveBackgroundsPlugin,
        audio::ThetawaveAudioPlugin,
        player::ThetawavePlayerPlugin,
        physics::ThetawavePhysicsPlugin,
        save::ThetawaveSavePlugin,
        core::ThetawaveCorePlugin,
    ));

    // plugins not used for wasm32 builds
    if !cfg!(target_arch = "wasm32") {
        app.add_plugins(window::ThetawaveWindowPlugin);
    }

    app.run();
}
