use std::path::PathBuf;

use bevy::{
    asset::{AssetPlugin, UnapprovedPathMode},
    prelude::*,
    window::{PresentMode, WindowResolution},
};

use thetawave_mob_editor::MobEditorPlugin;

/// Find the assets directory by searching up from the current working directory
fn find_assets_path() -> PathBuf {
    let cwd = std::env::current_dir().expect("Failed to get current working directory");

    // Try current directory first
    let assets_in_cwd = cwd.join("assets");
    if assets_in_cwd.join("media/aseprite").exists() {
        println!("Found assets at: {:?}", assets_in_cwd);
        return assets_in_cwd;
    }

    // Try parent directories
    let mut check_dir = cwd.clone();
    for _ in 0..3 {
        if let Some(parent) = check_dir.parent() {
            check_dir = parent.to_path_buf();
            let assets_path = check_dir.join("assets");
            if assets_path.join("media/aseprite").exists() {
                println!("Found assets at: {:?}", assets_path);
                return assets_path;
            }
        }
    }

    // Fallback - just use "assets" and hope for the best
    println!("Warning: Could not find assets directory, using 'assets'");
    PathBuf::from("assets")
}

/// Find or create the mods directory relative to the assets directory
fn find_mods_path(assets_path: &PathBuf) -> PathBuf {
    // Mods directory is a sibling of assets (e.g., assets/../mods)
    let mods_path = assets_path
        .parent()
        .map(|p| p.join("mods"))
        .unwrap_or_else(|| PathBuf::from("mods"));

    // Create mods/mobs directory if it doesn't exist
    let mods_mobs_path = mods_path.join("mobs");
    if !mods_mobs_path.exists() {
        if let Err(e) = std::fs::create_dir_all(&mods_mobs_path) {
            println!("Warning: Could not create mods directory: {}", e);
        } else {
            println!("Created mods directory at: {:?}", mods_mobs_path);
        }
    } else {
        println!("Found mods at: {:?}", mods_mobs_path);
    }

    mods_mobs_path
}

fn main() {
    // Determine the correct assets path before building the app
    let assets_path = find_assets_path();
    let mods_path = find_mods_path(&assets_path);
    println!("Using assets path: {:?}", assets_path);

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Thetawave Mob Editor".to_string(),
                        resolution: WindowResolution::new(1400, 900),
                        present_mode: PresentMode::AutoVsync,
                        resizable: true,
                        ..default()
                    }),
                    // Don't close automatically - let our handler show unsaved changes dialog
                    close_when_requested: false,
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                // SECURITY NOTE: UnapprovedPathMode::Allow
                //
                // This setting allows the Bevy asset system to load files from any path,
                // bypassing the default sandboxing. This is required for the mob editor because:
                //
                // 1. Editor loads assets from multiple directory trees:
                //    - Base assets: assets/mobs/
                //    - Game assets: thetawave-test-game/assets/mobs/
                //    - Mods assets: mods/mobs/
                //    - Sprite files: media/aseprite/
                //
                // 2. Security mitigations in place:
                //    - Only used in editor binary (not game binary)
                //    - File browser restricted to .mob/.mobpatch files only
                //    - Directory scanning limited to depth 10 to prevent infinite loops
                //    - Hidden files (starting with .) automatically filtered
                //    - Paths derived from filesystem scanning, not user text input
                //
                // 3. Risk assessment:
                //    - Low risk: Editor is a development tool, not end-user facing
                //    - Asset loading restricted to discovered directory trees
                //    - No network or remote path support
                .set(AssetPlugin {
                    file_path: assets_path.to_string_lossy().to_string(),
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                }),
        )
        .add_plugins(MobEditorPlugin {
            base_assets_dir: PathBuf::from("assets/mobs"),
            game_assets_dir: Some(PathBuf::from("thetawave-test-game/assets/mobs")),
            mods_assets_dir: Some(mods_path),
            show_base_mobs: true,
        })
        .run();
}
