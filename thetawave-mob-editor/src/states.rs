//! Editor state management.
//!
//! Defines the state machines that control the editor's behavior:
//! - [`EditorState`] - Main application states (Loading, Browsing, Editing)
//! - [`EditingMode`] - Sub-states for different editing panels
//! - [`DialogState`] - Modal dialog visibility states

use bevy::prelude::States;

/// Main application states for the mob editor
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum EditorState {
    /// Initial state - loading sprite assets
    #[default]
    Loading,
    /// Browsing files, no mob loaded
    Browsing,
    /// Actively editing a mob
    Editing,
}

/// Sub-states for editing mode - which panel/mode is active
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum EditingMode {
    /// General properties panel (name, health, spawnable, etc.)
    #[default]
    Properties,
}

/// Dialog states for modal windows
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum DialogState {
    /// No dialog open
    #[default]
    None,
}
