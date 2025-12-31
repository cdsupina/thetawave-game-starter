use bevy::prelude::*;

use crate::{data::EditorSession, preview::PreviewSettings, states::EditorState};

/// Color for collider outlines
const COLLIDER_COLOR: Color = Color::srgba(0.0, 1.0, 0.5, 0.8);
/// Color for selected collider
const SELECTED_COLLIDER_COLOR: Color = Color::srgba(1.0, 1.0, 0.0, 1.0);

/// Draw collider gizmos for the current mob
pub fn draw_collider_gizmos(
    mut gizmos: Gizmos,
    session: Res<EditorSession>,
    settings: Res<PreviewSettings>,
    state: Res<State<EditorState>>,
) {
    if *state.get() != EditorState::Editing || !settings.show_colliders {
        return;
    }

    // Use merged data for preview (falls back to current_mob for .mob files)
    let Some(mob) = session.mob_for_preview() else {
        return;
    };

    let Some(colliders) = mob.get("colliders").and_then(|v| v.as_array()) else {
        return;
    };

    for (index, collider) in colliders.iter().enumerate() {
        let Some(table) = collider.as_table() else {
            continue;
        };

        // Get position
        let position = table
            .get("position")
            .and_then(|v| v.as_array())
            .map(|arr| {
                let x = arr.first().and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                let y = arr.get(1).and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                Vec2::new(x, y)
            })
            .unwrap_or(Vec2::ZERO);

        // Get rotation
        let rotation = table
            .get("rotation")
            .and_then(|v| v.as_float())
            .unwrap_or(0.0) as f32;

        // Get shape
        let Some(shape) = table.get("shape").and_then(|v| v.as_table()) else {
            continue;
        };

        // Determine color based on selection
        let color = if session.selected_collider == Some(index) {
            SELECTED_COLLIDER_COLOR
        } else {
            COLLIDER_COLOR
        };

        // Draw the shape
        if let Some(circle_radius) = shape.get("Circle").and_then(|v| v.as_float()) {
            draw_circle_collider(&mut gizmos, position, circle_radius as f32, color);
        } else if let Some(rect_dims) = shape.get("Rectangle").and_then(|v| v.as_array()) {
            let width = rect_dims.first().and_then(|v| v.as_float()).unwrap_or(10.0) as f32;
            let height = rect_dims.get(1).and_then(|v| v.as_float()).unwrap_or(10.0) as f32;
            draw_rect_collider(&mut gizmos, position, width, height, rotation, color);
        }
    }
}

/// Draw a circle collider
fn draw_circle_collider(gizmos: &mut Gizmos, position: Vec2, radius: f32, color: Color) {
    gizmos.circle_2d(position, radius, color);

    // Draw cross at center
    let cross_size = radius * 0.3;
    gizmos.line_2d(
        position + Vec2::new(-cross_size, 0.0),
        position + Vec2::new(cross_size, 0.0),
        color,
    );
    gizmos.line_2d(
        position + Vec2::new(0.0, -cross_size),
        position + Vec2::new(0.0, cross_size),
        color,
    );
}

/// Draw a rectangle collider
fn draw_rect_collider(
    gizmos: &mut Gizmos,
    position: Vec2,
    width: f32,
    height: f32,
    rotation_deg: f32,
    color: Color,
) {
    let half_w = width / 2.0;
    let half_h = height / 2.0;

    // Calculate rotated corners
    let rotation = rotation_deg.to_radians();
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();

    let rotate_point = |x: f32, y: f32| -> Vec2 {
        Vec2::new(x * cos_r - y * sin_r, x * sin_r + y * cos_r) + position
    };

    let corners = [
        rotate_point(-half_w, -half_h),
        rotate_point(half_w, -half_h),
        rotate_point(half_w, half_h),
        rotate_point(-half_w, half_h),
    ];

    // Draw rectangle edges
    for i in 0..4 {
        gizmos.line_2d(corners[i], corners[(i + 1) % 4], color);
    }

    // Draw cross at center
    let cross_size = (width.min(height) * 0.2).max(2.0);
    gizmos.line_2d(
        position + Vec2::new(-cross_size, 0.0),
        position + Vec2::new(cross_size, 0.0),
        color,
    );
    gizmos.line_2d(
        position + Vec2::new(0.0, -cross_size),
        position + Vec2::new(0.0, cross_size),
        color,
    );
}

/// Draw a background grid
pub fn draw_grid(mut gizmos: Gizmos, settings: Res<PreviewSettings>) {
    if !settings.show_grid {
        return;
    }

    let grid_color = Color::srgba(0.3, 0.3, 0.3, 0.3);
    let axis_color = Color::srgba(0.5, 0.5, 0.5, 0.5);
    let grid_spacing = 10.0;
    let grid_extent = 200.0; // How far the grid extends

    // Calculate visible area based on zoom and pan
    let visible_extent = grid_extent / settings.zoom.max(0.5);

    // Draw grid lines
    let start = (-visible_extent / grid_spacing).floor() as i32;
    let end = (visible_extent / grid_spacing).ceil() as i32;

    for i in start..=end {
        let pos = i as f32 * grid_spacing;

        // Vertical lines
        if i == 0 {
            // Y axis
            gizmos.line_2d(
                Vec2::new(pos, -visible_extent),
                Vec2::new(pos, visible_extent),
                axis_color,
            );
        } else {
            gizmos.line_2d(
                Vec2::new(pos, -visible_extent),
                Vec2::new(pos, visible_extent),
                grid_color,
            );
        }

        // Horizontal lines
        if i == 0 {
            // X axis
            gizmos.line_2d(
                Vec2::new(-visible_extent, pos),
                Vec2::new(visible_extent, pos),
                axis_color,
            );
        } else {
            gizmos.line_2d(
                Vec2::new(-visible_extent, pos),
                Vec2::new(visible_extent, pos),
                grid_color,
            );
        }
    }
}

/// Draw spawner position indicators
pub fn draw_spawner_gizmos(
    mut gizmos: Gizmos,
    session: Res<EditorSession>,
    _settings: Res<PreviewSettings>,
    state: Res<State<EditorState>>,
) {
    if *state.get() != EditorState::Editing {
        return;
    }

    // Use merged data for preview (falls back to current_mob for .mob files)
    let Some(mob) = session.mob_for_preview() else {
        return;
    };

    // Draw projectile spawners
    if let Some(proj_spawners) = mob.get("projectile_spawners").and_then(|v| v.as_table())
        && let Some(spawners) = proj_spawners.get("spawners").and_then(|v| v.as_table()) {
            for (key, spawner) in spawners {
                let Some(s) = spawner.as_table() else {
                    continue;
                };

                let position = s
                    .get("position")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        let x = arr.first().and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        let y = arr.get(1).and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        Vec2::new(x, y)
                    })
                    .unwrap_or(Vec2::ZERO);

                let rotation = s
                    .get("rotation")
                    .and_then(|v| v.as_float())
                    .unwrap_or(0.0) as f32;

                // Draw a small triangle pointing in the spawn direction
                draw_spawner_indicator(&mut gizmos, position, rotation, Color::srgba(1.0, 0.5, 0.0, 0.8), key);
            }
        }

    // Draw mob spawners
    if let Some(mob_spawners) = mob.get("mob_spawners").and_then(|v| v.as_table())
        && let Some(spawners) = mob_spawners.get("spawners").and_then(|v| v.as_table()) {
            for (key, spawner) in spawners {
                let Some(s) = spawner.as_table() else {
                    continue;
                };

                let position = s
                    .get("position")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        let x = arr.first().and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        let y = arr.get(1).and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                        Vec2::new(x, y)
                    })
                    .unwrap_or(Vec2::ZERO);

                let rotation = s
                    .get("rotation")
                    .and_then(|v| v.as_float())
                    .unwrap_or(0.0) as f32;

                // Draw a diamond for mob spawners
                draw_spawner_indicator(&mut gizmos, position, rotation, Color::srgba(0.5, 0.0, 1.0, 0.8), key);
            }
        }
}

/// Draw a spawner direction indicator
fn draw_spawner_indicator(gizmos: &mut Gizmos, position: Vec2, rotation_deg: f32, color: Color, _key: &str) {
    let rotation = rotation_deg.to_radians();
    let size = 5.0;

    // Direction vector
    let dir = Vec2::new(rotation.cos(), rotation.sin());
    let perp = Vec2::new(-dir.y, dir.x);

    // Triangle pointing in spawn direction
    let tip = position + dir * size * 2.0;
    let base1 = position + perp * size;
    let base2 = position - perp * size;

    gizmos.line_2d(base1, tip, color);
    gizmos.line_2d(base2, tip, color);
    gizmos.line_2d(base1, base2, color);

    // Draw a small dot at the spawn position
    gizmos.circle_2d(position, 2.0, color);
}
