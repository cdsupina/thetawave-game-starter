//! Manual asset loading for when bevy_asset_loader is not available.
//!
//! This module parses the .assets.ron files and loads assets using AssetServer directly.

use bevy::{
    asset::{AssetServer, Handle},
    ecs::system::{Commands, Res, ResMut},
    image::Image,
    platform::collections::HashMap,
    scene::Scene,
    state::state::NextState,
    text::Font,
};
use bevy_aseprite_ultra::prelude::Aseprite;
use bevy_enoki::Particle2dEffect;
use bevy_kira_audio::AudioSource;
use ron::Value;
use thetawave_core::AppState;

use crate::data::{
    AssetFileStem, BackgroundAssets, ExtendedBackgroundAssets, ExtendedGameAssets,
    ExtendedMusicAssets, ExtendedUiAssets, GameAssets, MusicAssets, UiAssets,
};

// ============================================================================
// RON Parsing Utilities
// ============================================================================

/// Parses a .assets.ron file content and extracts the paths for a given key.
/// The format is: ({ "key": Files(paths: ["path1", "path2", ...]), ... })
fn parse_asset_paths(content: &str, key: &str) -> Vec<String> {
    // Parse the RON content
    let value: Value = match ron::from_str(content) {
        Ok(v) => v,
        Err(e) => {
            bevy::log::warn!("Failed to parse RON content: {}", e);
            return Vec::new();
        }
    };

    // The top level is a tuple containing a map: ({ ... })
    // So we need to unwrap the tuple first
    let map_value = match &value {
        Value::Seq(seq) if seq.len() == 1 => &seq[0],
        Value::Map(_) => &value,
        _ => {
            bevy::log::warn!("Unexpected RON structure: expected tuple or map at top level");
            return Vec::new();
        }
    };

    // Navigate to the key's paths
    if let Value::Map(map) = map_value {
        for (k, v) in map.iter() {
            if let Value::String(key_str) = k {
                if key_str == key {
                    return extract_paths_from_files_value(v);
                }
            }
        }
    }

    Vec::new()
}

/// Extracts paths from a Files(paths: [...]) value
fn extract_paths_from_files_value(value: &Value) -> Vec<String> {
    // The value is a named struct like Files(paths: [...])
    // RON represents this as a Map with field names as keys
    match value {
        Value::Map(map) => {
            for (k, v) in map.iter() {
                if let Value::String(key) = k {
                    if key == "paths" {
                        return extract_string_array(v);
                    }
                }
            }
            Vec::new()
        }
        // Also handle if it's directly a sequence
        Value::Seq(_) => extract_string_array(value),
        _ => Vec::new(),
    }
}

/// Extracts strings from a Value::Seq
fn extract_string_array(value: &Value) -> Vec<String> {
    if let Value::Seq(seq) = value {
        seq.iter()
            .filter_map(|v| {
                if let Value::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}

/// Extracts the file stem (filename without extension) from a path
fn extract_file_stem(path: &str) -> AssetFileStem {
    // Extract just the filename without extension to match bevy_asset_loader behavior
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| path.to_string())
}

/// Loads assets from paths into a HashMap
fn load_assets_mapped<T: bevy::asset::Asset>(
    paths: Vec<String>,
    asset_server: &AssetServer,
) -> HashMap<AssetFileStem, Handle<T>> {
    paths
        .into_iter()
        .map(|path| {
            let handle = asset_server.load(path.clone());
            let stem = extract_file_stem(&path);
            (stem, handle)
        })
        .collect()
}

/// Loads assets from paths into a Vec
fn load_assets_vec<T: bevy::asset::Asset>(
    paths: Vec<String>,
    asset_server: &AssetServer,
) -> Vec<Handle<T>> {
    paths
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect()
}

// ============================================================================
// Asset Loading Systems
// ============================================================================

/// System to load UI assets
pub fn load_ui_assets_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ron_content = include_str!("../../assets/ui.assets.ron");

    let sprites = load_assets_mapped::<Aseprite>(
        parse_asset_paths(ron_content, "ui_sprites"),
        &asset_server,
    );
    let images = load_assets_mapped::<Image>(
        parse_asset_paths(ron_content, "ui_images"),
        &asset_server,
    );
    let fonts = load_assets_mapped::<Font>(
        parse_asset_paths(ron_content, "ui_fonts"),
        &asset_server,
    );
    let menu_button_select_effects = load_assets_vec::<AudioSource>(
        parse_asset_paths(ron_content, "ui_button_select_audio"),
        &asset_server,
    );
    let menu_button_release_effects = load_assets_vec::<AudioSource>(
        parse_asset_paths(ron_content, "ui_button_release_audio"),
        &asset_server,
    );
    let menu_button_confirm_effects = load_assets_vec::<AudioSource>(
        parse_asset_paths(ron_content, "ui_button_confirm_audio"),
        &asset_server,
    );

    commands.insert_resource(UiAssets {
        sprites,
        images,
        fonts,
        menu_button_select_effects,
        menu_button_release_effects,
        menu_button_confirm_effects,
    });

    // Extended UI assets - empty for now since extended:// paths aren't supported
    commands.insert_resource(ExtendedUiAssets::default());
}

/// System to load music assets
pub fn load_music_assets_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ron_content = include_str!("../../assets/music.assets.ron");

    let music =
        load_assets_mapped::<AudioSource>(parse_asset_paths(ron_content, "music"), &asset_server);

    commands.insert_resource(MusicAssets { music });

    // Extended music assets - empty for now
    commands.insert_resource(ExtendedMusicAssets::default());
}

/// System to load background assets
pub fn load_background_assets_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ron_content = include_str!("../../assets/background.assets.ron");

    let space_backgrounds = load_assets_vec::<Image>(
        parse_asset_paths(ron_content, "space_backgrounds"),
        &asset_server,
    );
    let planets =
        load_assets_vec::<Scene>(parse_asset_paths(ron_content, "planets"), &asset_server);

    commands.insert_resource(BackgroundAssets {
        space_backgrounds,
        planets,
    });

    // Extended background assets - empty for now
    commands.insert_resource(ExtendedBackgroundAssets::default());
}

/// System to load game assets
pub fn load_game_assets_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ron_content = include_str!("../../assets/game.assets.ron");

    let sprites = load_assets_mapped::<Aseprite>(
        parse_asset_paths(ron_content, "game_sprites"),
        &asset_server,
    );
    let particle_effects = load_assets_mapped::<Particle2dEffect>(
        parse_asset_paths(ron_content, "game_particle_effects"),
        &asset_server,
    );

    commands.insert_resource(GameAssets {
        sprites,
        particle_effects,
    });

    // Extended game assets - empty for now
    commands.insert_resource(ExtendedGameAssets::default());
}

// ============================================================================
// State Transition Systems
// ============================================================================

/// Transition from MainMenuLoading to MainMenu
pub fn transition_to_main_menu(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::MainMenu);
}

/// Transition from GameLoading to Game
pub fn transition_to_game(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Game);
}
