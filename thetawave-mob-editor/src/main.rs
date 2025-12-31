use bevy::{
    asset::{AssetPlugin, UnapprovedPathMode},
    prelude::*,
    window::{PresentMode, WindowResolution},
};

use thetawave_mob_editor::MobEditorPlugin;

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
                // SECURITY NOTE: UnapprovedPathMode::Allow
                //
                // This setting allows the Bevy asset system to load files from any path,
                // bypassing the default sandboxing. This is required for the mob editor because:
                //
                // 1. Editor loads assets from multiple directory trees:
                //    - Base assets: assets/mobs/
                //    - Extended assets: thetawave-test-game/assets/mobs/
                //    - Sprite files: media/aseprite/
                //
                // 2. Security mitigations in place:
                //    - Only used in editor binary (not game binary)
                //    - File browser only shows .mob/.mobpatch files
                //    - Path validation rejects traversal characters (/, \, ..)
                //    - No arbitrary path input - all paths derived from scanned directories
                //
                // 3. Risk assessment:
                //    - Low risk: Editor is a development tool, not end-user facing
                //    - Asset loading restricted to discovered directory trees
                //    - No network or remote path support
                .set(AssetPlugin {
                    file_path: assets_path,
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                }),
        )
        .add_plugins(MobEditorPlugin::default())
        .run();
}
