use bevy::{
    ecs::{error::Result, query::With},
    prelude::Query,
    window::{PrimaryWindow, Window},
};
use bevy_egui::{
    EguiContextSettings, EguiContexts,
    egui::{
        Color32, CornerRadius, Spacing, Stroke, Style, Vec2, Visuals,
        style::{HandleShape, Selection, WidgetVisuals, Widgets},
    },
};

const HOVERED_BG_COLOR: Color32 = Color32::from_rgb(226, 114, 133);
const HOVERED_STROKE: Stroke = Stroke {
    width: 2.0,
    color: Color32::from_rgb(246, 162, 168),
};
const INACTIVE_BG_COLOR: Color32 = Color32::from_rgb(178, 82, 102);
const INACTIVE_STROKE: Stroke = Stroke {
    width: 2.0,
    color: Color32::from_rgb(226, 114, 133),
};
const WHITE_STROKE: Stroke = Stroke {
    width: 2.0,
    color: Color32::WHITE,
};
const ACTIVE_BG_COLOR: Color32 = Color32::from_rgb(139, 68, 89);
const ACTIVE_STROKE: Stroke = Stroke {
    width: 2.0,
    color: Color32::from_rgb(178, 82, 102),
};
const SELECTION_BG_COLOR: Color32 = Color32::from_rgb(139, 68, 89);

/// Sets the scale of the egui elements
pub(in crate::ui) fn setup_egui_system(
    mut egui_settings: Query<&mut EguiContextSettings>,
    mut contexts: EguiContexts,
) -> Result {
    // Increase scale of egui options menu
    egui_settings.single_mut()?.scale_factor = 2.0;

    // Customize style of egui
    contexts.ctx_mut()?.set_style(Style {
        visuals: Visuals {
            //dark_mode: true,
            slider_trailing_fill: true,
            handle_shape: HandleShape::Rect { aspect_ratio: 0.5 },
            widgets: Widgets {
                hovered: WidgetVisuals {
                    bg_fill: HOVERED_BG_COLOR,
                    weak_bg_fill: HOVERED_BG_COLOR,
                    bg_stroke: HOVERED_STROKE,
                    corner_radius: CornerRadius::same(2),
                    fg_stroke: WHITE_STROKE,
                    expansion: 1.2,
                },
                inactive: WidgetVisuals {
                    bg_fill: INACTIVE_BG_COLOR,
                    weak_bg_fill: INACTIVE_BG_COLOR,
                    bg_stroke: INACTIVE_STROKE,
                    corner_radius: CornerRadius::same(2),
                    fg_stroke: WHITE_STROKE,
                    expansion: 1.0,
                },
                active: WidgetVisuals {
                    bg_fill: ACTIVE_BG_COLOR,
                    weak_bg_fill: ACTIVE_BG_COLOR,
                    bg_stroke: ACTIVE_STROKE,
                    corner_radius: CornerRadius::same(2),
                    fg_stroke: WHITE_STROKE,
                    expansion: 1.0,
                },
                ..Default::default()
            },
            override_text_color: Some(Color32::WHITE),
            selection: Selection {
                bg_fill: SELECTION_BG_COLOR,
                ..Default::default()
            },
            ..Default::default()
        },
        spacing: Spacing {
            item_spacing: Vec2::new(12.0, 12.0),
            slider_width: 100.0,
            slider_rail_height: 15.0,
            ..Default::default()
        },
        ..Default::default()
    });

    Ok(())
}

/// System that updates egui scale based on window height
pub fn update_egui_scale_system(
    mut egui_settings: Query<&mut EguiContextSettings>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = primary_window_q.single() {
        // Calculate egui scale based on physical window height relative to 720p baseline
        if let Ok(mut egui_settings) = egui_settings.single_mut() {
            egui_settings.scale_factor = (2. / 720.) * (window.resolution.physical_height() as f32);
        }
    }
}
