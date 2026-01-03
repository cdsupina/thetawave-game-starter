//! File system operations for the editor.
//!
//! - [`FileTreeState`] - File browser tree state
//! - [`FileOperations`] - Load, save, and delete operations
//! - Asset registration helpers for `game.assets.ron`

mod assets_ron_parser;
mod browser;
mod operations;

pub(crate) use assets_ron_parser::{append_sprite_to_assets_ron, parse_assets_ron};
pub(crate) use browser::{FileNode, FileTreeState};
pub(crate) use operations::{
    DeleteDirectoryEvent, DeleteMobEvent, FileOperations, LoadMobEvent, NewMobEvent,
    ReloadMobEvent, SaveMobEvent, merge_toml_values,
};
