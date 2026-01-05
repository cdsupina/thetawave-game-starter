// Re-export bevy so games don't need to add it as a separate dependency
pub use bevy;

use bevy::{
    DefaultPlugins,
    app::Plugin,
    asset::{AssetMetaCheck, AssetPlugin},
    ecs::{
        entity::Entity,
        system::{In, SystemId},
    },
    image::ImagePlugin,
    input::keyboard::KeyCode,
    prelude::PluginGroup,
    utils::default,
    window::{Window, WindowMode, WindowPlugin, WindowResolution},
};

#[cfg(not(target_arch = "wasm32"))]
use bevy::asset::{AssetApp, io::AssetSource, io::file::FileAssetReader};

#[cfg(target_arch = "wasm32")]
use bevy::asset::{AssetApp, io::AssetSource, io::wasm::HttpWasmAssetReader};

use bevy_aseprite_ultra::AsepriteUltraPlugin;

mod embedded_reader;

use bevy_platform::collections::{HashMap, HashSet};
#[cfg(feature = "debug")]
use thetawave_debug::ThetawaveDebugPlugin;

mod audio;
mod camera;
mod collisions;
mod options;
mod player;
mod save;
mod states;
mod ui;
mod window;

pub struct ThetawaveStarterPlugin {
    pub window_title: String,
    pub starting_resolution: (u32, u32),
    pub show_debug_keycode: KeyCode,
    pub extended_abilities: HashMap<String, SystemId<In<Entity>>>,
    pub extended_duration_abilities: HashSet<String>,
}

impl Plugin for ThetawaveStarterPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        // Tier 2: Game assets (developer's assets folder, relative to Cargo.toml)
        #[cfg(not(target_arch = "wasm32"))]
        app.register_asset_source(
            "game",
            AssetSource::build().with_reader(|| Box::new(FileAssetReader::new("assets"))),
        );

        #[cfg(target_arch = "wasm32")]
        app.register_asset_source(
            "game",
            AssetSource::build().with_reader(|| Box::new(HttpWasmAssetReader::new("assets"))),
        );

        // Tier 3: Mod assets (relative to executable, never embedded)
        // Only register if directory setup succeeds
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(mods_path) = setup_mods_directory() {
            app.register_asset_source(
                "mods",
                AssetSource::build()
                    .with_reader(move || Box::new(FileAssetReader::new(mods_path.clone()))),
            );
        }

        #[cfg(target_arch = "wasm32")]
        app.register_asset_source(
            "mods",
            AssetSource::build().with_reader(|| Box::new(HttpWasmAssetReader::new("mods"))),
        );

        // Register embedded base assets as the default source (before DefaultPlugins)
        app.register_asset_source(
            bevy::asset::io::AssetSourceId::Default,
            embedded_reader::embedded_asset_source(),
        );

        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()) // necessary for crisp pixel art
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        mode: WindowMode::Windowed,
                        resolution: WindowResolution::new(
                            self.starting_resolution.0,
                            self.starting_resolution.1,
                        )
                        .with_scale_factor_override(1.0),
                        resizable: false,
                        title: self.window_title.clone(),
                        // For wasm: use the canvas defined in index.html
                        #[cfg(target_arch = "wasm32")]
                        canvas: Some("#bevy".to_string()),
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
            player::ThetawavePlayerPlugin {
                extended_abilities: self.extended_abilities.clone(),
                extended_duration_abilities: self.extended_duration_abilities.clone(),
            },
            thetawave_physics::ThetawavePhysicsPlugin,
            save::ThetawaveSavePlugin,
            states::ThetawaveStatesPlugin,
            thetawave_mobs::ThetawaveMobsPlugin,
            thetawave_projectiles::ThetawaveProjectilesPlugin,
        ));

        app.add_plugins((
            thetawave_core::ThetawaveCorePlugin,
            collisions::ThetawaveCollisionsPlugin,
            thetawave_particles::ThetawaveParticlesPlugin,
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

/// Set up the mods directory and create default asset files if needed.
///
/// Returns the mods path if setup succeeds, None if it fails.
/// This ensures the mods:// asset source is only registered when usable.
///
/// Note: This runs on every startup, but the exists() checks make the overhead
/// negligible (~1 syscall per file when files already exist).
#[cfg(not(target_arch = "wasm32"))]
fn setup_mods_directory() -> Option<std::path::PathBuf> {
    use std::fs;

    let mods_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("mods")))?;

    // Create the mods directory if it doesn't exist
    if !mods_path.exists() {
        if let Err(e) = fs::create_dir_all(&mods_path) {
            bevy::log::warn!("Failed to create mods directory: {}", e);
            return None;
        }
    }

    // Default content for each .assets.ron file
    let asset_files = [
        (
            "ui.assets.ron",
            r#"({
    "mod_ui_sprites": Files(paths: []),
    "mod_ui_images": Files(paths: []),
    "mod_ui_fonts": Files(paths: []),
    "mod_ui_button_select_audio": Files(paths: []),
    "mod_ui_button_release_audio": Files(paths: []),
    "mod_ui_button_confirm_audio": Files(paths: []),
})"#,
        ),
        (
            "music.assets.ron",
            r#"({
    "mod_music": Files(paths: []),
})"#,
        ),
        (
            "background.assets.ron",
            r#"({
    "mod_space_backgrounds": Files(paths: []),
    "mod_planets": Files(paths: []),
})"#,
        ),
        (
            "game.assets.ron",
            r#"({
    "mod_game_sprites": Files(paths: []),
    "mod_game_particle_effects": Files(paths: []),
})"#,
        ),
        (
            "mobs.assets.ron",
            r#"({
    "mod_mobs": Files(paths: []),
    "mod_mob_patches": Files(paths: []),
})"#,
        ),
    ];

    for (filename, content) in asset_files {
        let file_path = mods_path.join(filename);
        if !file_path.exists() {
            if let Err(e) = fs::write(&file_path, content) {
                bevy::log::warn!("Failed to create {}: {}", filename, e);
                // Continue anyway - partial setup is better than none
            }
        }
    }

    Some(mods_path)
}
