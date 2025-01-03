use bevy::{
    app::App,
    prelude::{DefaultPlugins, ImagePlugin, PluginGroup},
    utils::default,
    window::{Window, WindowMode, WindowPlugin, WindowResolution},
};
use bevy_aseprite_ultra::AsepriteUltraPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod assets;
mod audio;
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
            .set(ImagePlugin::default_nearest()) // necessary for crisp pixel art
            .set(WindowPlugin {
                primary_window: Some(Window {
                    mode: WindowMode::Windowed,
                    resolution: WindowResolution::new(1280.0, 720.0)
                        .with_scale_factor_override(1.0),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
        AsepriteUltraPlugin, // plugin for using Aseprite assets
        // custom plugins for Thetawave
        audio::ThetawaveAudioPlugin,
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
