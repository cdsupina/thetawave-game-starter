use bevy::{
    app::App,
    prelude::{DefaultPlugins, ImagePlugin, PluginGroup},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod assets;
mod backgrounds;
mod camera;
mod input;
mod states;
mod ui;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        assets::ThetawaveAssetsPlugin, // must be registered before AssetPlugin due to EmbeddedAssetPlugin
        DefaultPlugins.set(ImagePlugin::default_nearest()), // necessary for crisp pixel art
        // custom plugins for Thetawave
        ui::ThetawaveUiPlugin,
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
