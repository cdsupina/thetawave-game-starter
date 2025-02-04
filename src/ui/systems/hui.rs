use bevy::prelude::Query;
use bevy_egui::EguiContextSettings;

/// This function sets up the main menu user interface. It spawns the main menu HTML node and registers the required functions and components.
pub(in crate::ui) fn setup_hui_system(mut egui_settings: Query<&mut EguiContextSettings>) {
    // Increase scale of egui options menu
    if !cfg!(feature = "world_inspector") {
        egui_settings.single_mut().scale_factor = 2.0;
    }
}
