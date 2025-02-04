use crate::ui::data::{ButtonAction, UiChildBuilderExt};

use super::{ApplyOptionsEvent, Cleanup, MainMenuState, OptionsRes, UiAssets};
use bevy::{
    core::Name,
    hierarchy::{BuildChildren, ChildBuild},
    log::info,
    prelude::{Commands, EventReader, Res, ResMut},
    ui::{AlignItems, Display, FlexDirection, JustifyContent, Node, UiRect, Val},
    utils::default,
    window::{MonitorSelection, WindowMode, WindowResolution},
};
use bevy_egui::{egui, EguiContexts};
use bevy_persistent::Persistent;

/// Spawns options menu ui for the main menu
pub(in crate::ui) fn spawn_options_menu_system(mut cmds: Commands, ui_assets: Res<UiAssets>) {
    cmds.spawn((
        Cleanup::<MainMenuState> {
            states: vec![MainMenuState::Options],
        },
        Name::new("Options Menu"),
        // Top level parent node
        Node {
            height: Val::Percent(100.0),
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            flex_direction: FlexDirection::Column,
            ..default()
        },
    ))
    .with_children(|parent| {
        parent
            .spawn(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_menu_button(
                    &ui_assets,
                    ButtonAction::ApplyOptions,
                    300.0,
                    true,
                    false,
                );
                parent.spawn_menu_button(
                    &ui_assets,
                    ButtonAction::EnterMainMenuState(MainMenuState::Title),
                    300.0,
                    false,
                    false,
                );
            });
    });
}

/// This function is a system that handles the egui options menu
pub(in crate::ui) fn options_menu_system(
    mut contexts: EguiContexts,
    mut options_res: ResMut<Persistent<OptionsRes>>,
) {
    egui::CentralPanel::default()
        .frame(egui::Frame {
            fill: egui::Color32::TRANSPARENT,       // Set transparent background
            inner_margin: egui::Margin::same(10.0), // Establish inner margin for UI layout
            ..Default::default()
        })
        .show(contexts.ctx_mut(), |ui| {
            // Combo box for selecting Window Mode.
            ui.horizontal(|ui| {
                ui.label("Window Mode");
                egui::ComboBox::from_id_salt("window_mode_combobox")
                    .selected_text(window_mode_to_string(&options_res.window_mode).to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut options_res.window_mode,
                            WindowMode::Windowed,
                            "Windowed",
                        );
                        ui.selectable_value(
                            &mut options_res.window_mode,
                            WindowMode::Fullscreen(MonitorSelection::Current),
                            "Fullscreen",
                        );
                    });
            });

            // Combo box for selecting screen resolution.
            ui.horizontal(|ui| {
                ui.label("Resolution");
                egui::ComboBox::from_id_salt("resolution_combobox")
                    .selected_text(
                        window_resolution_to_string(&options_res.window_resolution).to_string(),
                    )
                    .show_ui(ui, |ui| {
                        // Iterate through every available resolution and create a selectable value
                        for resolution in options_res.get_resolutions() {
                            ui.selectable_value(
                                &mut options_res.window_resolution,
                                resolution.clone(),
                                window_resolution_to_string(&resolution),
                            );
                        }
                    });
            });

            // Volume sliders
            ui.horizontal(|ui| {
                ui.label("Master Volume");
                ui.add(egui::Slider::new(&mut options_res.master_volume, 0.0..=1.0));
            });

            ui.horizontal(|ui| {
                ui.label("Music Volume");
                ui.add(egui::Slider::new(&mut options_res.music_volume, 0.0..=1.0));
            });

            ui.horizontal(|ui| {
                ui.label("Effects Volume");
                ui.add(egui::Slider::new(
                    &mut options_res.effects_volume,
                    0.0..=1.0,
                ));
            });

            ui.horizontal(|ui| {
                ui.label("Ui Volume");
                ui.add(egui::Slider::new(&mut options_res.ui_volume, 0.0..=1.0));
            });
        });
}

/// Converts WindowMode enum to a string representation
/// Returns a string slice describing the window mode (e.g. "Windowed", "Fullscreen", etc.)
fn window_mode_to_string(mode: &WindowMode) -> &str {
    match mode {
        WindowMode::Windowed => "Windowed",
        WindowMode::BorderlessFullscreen(_) => "Borderless Fullscreen",
        WindowMode::Fullscreen(_) => "Fullscreen",
        WindowMode::SizedFullscreen(_) => "Sized Fullscreen",
    }
}

/// Converts WindowResolution to a formatted string
/// Takes a WindowResolution reference and returns a string in the format "WIDTHxHEIGHT"
fn window_resolution_to_string(resolution: &WindowResolution) -> String {
    let res_vec = resolution.size();
    format!("{}x{}", res_vec.x, res_vec.y)
}

/// Save options to file when options applied
pub(in crate::ui) fn persist_options_system(
    options_res: Res<Persistent<OptionsRes>>,
    mut apply_options_events: EventReader<ApplyOptionsEvent>,
) {
    for _event in apply_options_events.read() {
        info!("Persisting options");
        options_res.persist().expect("failed to save new options");
    }
}
