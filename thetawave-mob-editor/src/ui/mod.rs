//! UI components for the mob editor.
//!
//! - File browser panel (left)
//! - Properties panel (right)
//! - Preview controls and status bar
//! - Modal dialogs for file operations

mod file_panel;
mod layout;
mod properties_panel;

use bevy_egui::egui;

/// Standard delete button background color (dark red)
pub(crate) const DELETE_BUTTON_COLOR: egui::Color32 = egui::Color32::from_rgb(120, 60, 60);

pub(crate) use file_panel::{
    DeleteDialogState, DeleteDirectoryDialogState, NewFolderDialog, NewMobDialog, UnsavedAction,
    UnsavedChangesDialog, file_panel_ui, render_delete_dialog, render_delete_directory_dialog,
};
pub(crate) use layout::main_ui_system;
pub(crate) use properties_panel::{
    PropertiesPanelResult, properties_panel_ui, update_decoration_sprite,
};

/// Truncate a filename for display, keeping the extension visible
pub(crate) fn truncate_filename(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else if let Some(dot_pos) = name.rfind('.') {
        let ext = &name[dot_pos..];
        let available = max_len.saturating_sub(ext.len() + 2);
        if available > 3 {
            format!("{}..{}", &name[..available], ext)
        } else {
            format!("{}...", &name[..max_len.saturating_sub(3)])
        }
    } else {
        format!("{}...", &name[..max_len.saturating_sub(3)])
    }
}
