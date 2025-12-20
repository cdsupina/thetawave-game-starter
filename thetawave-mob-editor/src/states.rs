use bevy::prelude::*;

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
    /// Collider shape editing
    Colliders,
    /// Behavior tree editing
    Behavior,
    /// Projectile/Mob spawner configuration
    Spawners,
    /// Jointed mob configuration
    Joints,
}

/// Dialog states for modal windows
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum DialogState {
    /// No dialog open
    #[default]
    None,
    /// Creating a new mob file
    NewFile,
    /// Save As dialog
    SaveAs,
    /// Confirm delete dialog
    ConfirmDelete,
    /// Unsaved changes warning
    UnsavedChanges,
}
