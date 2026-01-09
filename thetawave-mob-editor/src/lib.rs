//! Thetawave Mob Editor
//!
//! A visual editor for creating and editing `.mob` files for thetawave-based games.
//!
//! # Usage
//!
//! Add the `MobEditorPlugin` to your Bevy app:
//!
//! ```rust,ignore
//! use std::path::PathBuf;
//! use bevy::prelude::*;
//! use thetawave_mob_editor::MobEditorPlugin;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(MobEditorPlugin {
//!         base_assets_dir: PathBuf::from("assets/mobs"),
//!         game_assets_dir: Some(PathBuf::from("my-game/assets/mobs")),
//!         mods_assets_dir: None, // Optional: mods directory
//!         show_base_mobs: true,
//!     })
//!     .run();
//! ```

pub(crate) mod data;
pub(crate) mod file;
pub mod plugin;
pub(crate) mod preview;
pub(crate) mod states;
pub(crate) mod ui;

// Re-export the main plugin for convenience
pub use plugin::{EditorConfig, MobEditorPlugin};

// Re-export bevy so games don't need to add it as a separate dependency
pub use bevy;
