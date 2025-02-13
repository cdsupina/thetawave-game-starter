use bevy::prelude::Query;
use bevy_egui::{
    egui::{
        style::{HandleShape, Selection, WidgetVisuals, Widgets},
        Color32, Rounding, Spacing, Stroke, Style, Vec2, Visuals,
    },
    EguiContextSettings, EguiContexts,
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
const SELECTION_BG_COLOR: Color32 = Color32::from_rgb(139, 68, 89);

/// Sets the scale of the egui elements
pub(in crate::ui) fn setup_egui_system(
    mut egui_settings: Query<&mut EguiContextSettings>,
    mut contexts: EguiContexts,
) {
    if let Ok(mut egui_settings) = egui_settings.get_single_mut() {
        // Increase scale of egui options menu
        egui_settings.scale_factor = 2.0;

        // Customize style of egui
        contexts.ctx_mut().set_style(Style {
            visuals: Visuals {
                //dark_mode: true,
                slider_trailing_fill: true,
                handle_shape: HandleShape::Rect { aspect_ratio: 0.5 },
                widgets: Widgets {
                    hovered: WidgetVisuals {
                        bg_fill: HOVERED_BG_COLOR,
                        weak_bg_fill: HOVERED_BG_COLOR,
                        bg_stroke: HOVERED_STROKE,
                        rounding: Rounding::same(2.0),
                        fg_stroke: HOVERED_STROKE,
                        expansion: 1.2,
                    },
                    inactive: WidgetVisuals {
                        bg_fill: INACTIVE_BG_COLOR,
                        weak_bg_fill: INACTIVE_BG_COLOR,
                        bg_stroke: INACTIVE_STROKE,
                        rounding: Rounding::same(2.0),
                        fg_stroke: INACTIVE_STROKE,
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
    }
}
