use bevy::{
    asset::{AssetPlugin, UnapprovedPathMode},
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use std::path::PathBuf;

mod data;
mod file;
mod plugin;
mod preview;
mod states;
mod ui;

use plugin::MobEditorPlugin;

/// Find the assets directory by searching up from the current working directory
fn find_assets_path() -> String {
    let cwd = std::env::current_dir().unwrap_or_default();

    // Try current directory first
    let assets_in_cwd = cwd.join("assets");
    if assets_in_cwd.join("media/aseprite").exists() {
        println!("Found assets at: {:?}", assets_in_cwd);
        return assets_in_cwd.to_string_lossy().to_string();
    }

    // Try parent directories
    let mut check_dir = cwd.clone();
    for _ in 0..3 {
        if let Some(parent) = check_dir.parent() {
            check_dir = parent.to_path_buf();
            let assets_path = check_dir.join("assets");
            if assets_path.join("media/aseprite").exists() {
                println!("Found assets at: {:?}", assets_path);
                return assets_path.to_string_lossy().to_string();
            }
        }
    }

    // Fallback - just use "assets" and hope for the best
    println!("Warning: Could not find assets directory, using 'assets'");
    "assets".to_string()
}

fn main() {
    // Determine the correct assets path before building the app
    let assets_path = find_assets_path();
    println!("Using assets path: {}", assets_path);

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
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                // Allow loading from absolute paths (needed for both base and extended assets)
                .set(AssetPlugin {
                    file_path: assets_path,
                    // Allow loading from any path - needed for editor to load from multiple directories
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                }),
        )
        .add_plugins(MobEditorPlugin::default())
        .run();
}
