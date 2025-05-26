use bevy::{
    app::Plugin,
    prelude::PluginGroup,
    render::texture::ImagePlugin,
    utils::default,
    window::{Window, WindowMode, WindowPlugin, WindowResolution},
    DefaultPlugins,
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

pub struct ThetawaveStarterPlugin {
    pub window_title: String,
    pub starting_resolution: (f32, f32),
}

impl Plugin for ThetawaveStarterPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            assets::ThetawaveAssetsPlugin, // must be registered before AssetPlugin due to EmbeddedAssetPlugin
            DefaultPlugins
                .set(ImagePlugin::default_nearest()) // necessary for crisp pixel art
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        mode: WindowMode::Windowed,
                        resolution: WindowResolution::from(self.starting_resolution)
                            .with_scale_factor_override(1.0),
                        resizable: false,
                        title: self.window_title.clone(),
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
    }
}
