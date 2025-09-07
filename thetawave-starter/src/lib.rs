use bevy::{
    DefaultPlugins,
    app::Plugin,
    asset::{AssetMetaCheck, AssetPlugin},
    input::keyboard::KeyCode,
    prelude::PluginGroup,
    render::texture::ImagePlugin,
    utils::default,
    window::{Window, WindowMode, WindowPlugin, WindowResolution},
};

#[cfg(not(target_arch = "wasm32"))]
use bevy::asset::{AssetApp, io::AssetSource, io::file::FileAssetReader};

#[cfg(target_arch = "wasm32")]
use bevy::asset::{AssetApp, io::AssetSource, io::wasm::HttpWasmAssetReader};

use bevy_aseprite_ultra::AsepriteUltraPlugin;

use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};

#[cfg(feature = "debug")]
use thetawave_debug::ThetawaveDebugPlugin;

mod audio;
mod camera;
mod collisions;
mod options;
mod particles;
mod player;
mod save;
mod states;
mod ui;
mod window;

pub struct ThetawaveStarterPlugin {
    pub window_title: String,
    pub starting_resolution: (f32, f32),
    pub show_debug_keycode: KeyCode,
}

impl Plugin for ThetawaveStarterPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        #[cfg(not(target_arch = "wasm32"))]
        app.register_asset_source(
            "extended",
            AssetSource::build().with_reader(|| Box::new(FileAssetReader::new("assets"))),
        );

        #[cfg(target_arch = "wasm32")]
        app.register_asset_source(
            "extended",
            AssetSource::build().with_reader(|| Box::new(HttpWasmAssetReader::new("assets"))),
        );

        app.add_plugins((
            EmbeddedAssetPlugin {
                mode: PluginMode::ReplaceDefault, //embeds assets into binary
            },
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
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
            thetawave_assets::ThetawaveAssetsPlugin,
            AsepriteUltraPlugin, // plugin for using Aseprite assets
            // custom plugins for Thetawave
            ui::ThetawaveUiPlugin,
            options::ThetawaveOptionsPlugin,
            camera::ThetawaveCameraPlugin,
            thetawave_backgrounds::ThetawaveBackgroundsPlugin,
            audio::ThetawaveAudioPlugin,
            player::ThetawavePlayerPlugin,
            thetawave_physics::ThetawavePhysicsPlugin,
            save::ThetawaveSavePlugin,
            states::ThetawaveStatesPlugin,
            thetawave_mobs::ThetawaveMobsPlugin,
            thetawave_projectiles::ThetawaveProjectilesPlugin,
        ));

        app.add_plugins((
            thetawave_core::ThetawaveCorePlugin,
            collisions::ThetawaveCollisionsPlugin,
            particles::ThetawaveParticlesPlugin,
        ));

        // plugins only used in debug builds
        #[cfg(feature = "debug")]
        app.add_plugins(ThetawaveDebugPlugin {
            show_debug_keycode: self.show_debug_keycode,
        });

        // plugins not used for wasm32 builds
        if !cfg!(target_arch = "wasm32") {
            app.add_plugins(window::ThetawaveWindowPlugin);
        }
    }
}
