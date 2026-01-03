//! Data management for the editor.
//!
//! - [`EditorSession`] - Current editing session state
//! - [`SpriteRegistry`] - Registry of available sprites

mod session;
mod sprite_registry;

pub(crate) use session::{EditorSession, FileType};
pub(crate) use sprite_registry::{RegisteredSprite, SpriteRegistry, SpriteSource};
