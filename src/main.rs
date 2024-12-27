use bevy::{
    app::App,
    prelude::{DefaultPlugins, ImagePlugin, PluginGroup},
    utils::default,
    window::{Window, WindowMode, WindowPlugin, WindowResolution},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod assets;
mod backgrounds;
mod camera;
mod input;
mod options;
mod states;
mod ui;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        assets::ThetawaveAssetsPlugin, // must be registered before AssetPlugin due to EmbeddedAssetPlugin
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    mode: WindowMode::Windowed,
                    resolution: WindowResolution::new(1280.0, 720.0),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }), // necessary for crisp pixel art
        // custom plugins for Thetawave
        ui::ThetawaveUiPlugin,
        options::ThetawaveOptionsPlugin,
        input::ThetawaveInputPlugin,
        states::ThetawaveStatesPlugin,
        camera::ThetawaveCameraPlugin,
        backgrounds::ThetawaveBackgroundsPlugin,
    ));

    if cfg!(feature = "world_inspector") {
        app.add_plugins(WorldInspectorPlugin::new());
    }

    app.run();
}
