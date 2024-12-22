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
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()), // necessary for crisp pixel art
            // custom plugins for Thetawave
            ui::ThetawaveUiPlugin,
            input::ThetawaveInputPlugin,
            states::ThetawaveStatesPlugin,
            camera::ThetawaveCameraPlugin,
            assets::ThetawaveAssetsPlugin,
            backgrounds::ThetawaveBackgroundsPlugin,
            // plugin for inspecting entiies
            WorldInspectorPlugin::new(),
        ))
        .run();
}
