use bevy::prelude::{Alpha, Color, Gizmos, Res, State, Vec2};

use crate::{data::EditorSession, preview::PreviewSettings, states::EditorState};

/// Color for joint anchor points and connections
const JOINT_COLOR: Color = Color::srgba(0.5, 0.8, 1.0, 0.9);
/// Color for selected joint
const SELECTED_JOINT_COLOR: Color = Color::srgba(1.0, 1.0, 0.0, 1.0);
/// Color for angle limit arcs
const ANGLE_ARC_COLOR: Color = Color::srgba(1.0, 0.6, 0.2, 0.6);
/// Color for chain indicators
const CHAIN_COLOR: Color = Color::srgba(0.8, 0.4, 1.0, 0.7);

/// Draw joint gizmos for jointed mobs
pub fn draw_joint_gizmos(
    mut gizmos: Gizmos,
    session: Res<EditorSession>,
    settings: Res<PreviewSettings>,
    state: Res<State<EditorState>>,
) {
    if *state.get() != EditorState::Editing || !settings.show_joint_gizmos {
        return;
    }

    let Some(mob) = session.mob_for_preview() else {
        return;
    };

    let Some(jointed_mobs) = mob.get("jointed_mobs").and_then(|v| v.as_array()) else {
        return;
    };

    for (index, jointed) in jointed_mobs.iter().enumerate() {
        let Some(table) = jointed.as_table() else {
            continue;
        };

        // Determine if this joint is selected
        let is_selected = session.selected_jointed_mob == Some(index);
        let primary_color = if is_selected {
            SELECTED_JOINT_COLOR
        } else {
            JOINT_COLOR
        };

        // Parse positions
        let offset_pos = parse_vec2(table.get("offset_pos")).unwrap_or(Vec2::ZERO);
        let anchor_1_pos = parse_vec2(table.get("anchor_1_pos")).unwrap_or(Vec2::ZERO);
        let anchor_2_pos = parse_vec2(table.get("anchor_2_pos")).unwrap_or(Vec2::ZERO);

        // Draw parent anchor point (on parent mob, relative to origin)
        draw_anchor_point(&mut gizmos, anchor_1_pos, primary_color, 3.0);

        // Draw line from parent origin to child offset position
        gizmos.line_2d(Vec2::ZERO, offset_pos, primary_color.with_alpha(0.4));

        // Draw child position marker
        gizmos.circle_2d(offset_pos, 2.0, primary_color);

        // Draw child anchor point (relative to child, which is at offset_pos)
        let child_anchor_world = offset_pos + anchor_2_pos;
        draw_anchor_point(&mut gizmos, child_anchor_world, primary_color, 2.5);

        // Draw connection line between anchors
        gizmos.line_2d(anchor_1_pos, child_anchor_world, primary_color);

        // Draw angle limits if present
        if let Some(angle_table) = table.get("angle_limit_range").and_then(|v| v.as_table()) {
            let min_deg = angle_table
                .get("min")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0) as f32;
            let max_deg = angle_table
                .get("max")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0) as f32;
            draw_angle_arc(&mut gizmos, anchor_1_pos, min_deg, max_deg, ANGLE_ARC_COLOR);
        }

        // Draw chain indicator if present
        if let Some(chain_table) = table.get("chain").and_then(|v| v.as_table()) {
            let length = chain_table
                .get("length")
                .and_then(|v| v.as_integer())
                .unwrap_or(1) as usize;
            let pos_offset = parse_vec2(chain_table.get("pos_offset")).unwrap_or(Vec2::ZERO);
            draw_chain_indicator(&mut gizmos, offset_pos, pos_offset, length, CHAIN_COLOR);
        }
    }
}

/// Draw an anchor point marker (small circle with cross)
fn draw_anchor_point(gizmos: &mut Gizmos, position: Vec2, color: Color, radius: f32) {
    gizmos.circle_2d(position, radius, color);

    // Draw small cross
    let cross_size = radius * 0.6;
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

/// Draw an arc showing angle limits
fn draw_angle_arc(gizmos: &mut Gizmos, center: Vec2, min_deg: f32, max_deg: f32, color: Color) {
    let arc_radius = 8.0;

    // Joints typically point downward, so offset by -90 degrees
    let base_angle = -std::f32::consts::FRAC_PI_2;
    let min_rad = base_angle + min_deg.to_radians();
    let max_rad = base_angle + max_deg.to_radians();

    // Draw lines showing angle limits
    let min_end = center + Vec2::new(min_rad.cos(), min_rad.sin()) * arc_radius;
    let max_end = center + Vec2::new(max_rad.cos(), max_rad.sin()) * arc_radius;

    gizmos.line_2d(center, min_end, color);
    gizmos.line_2d(center, max_end, color);

    // Draw arc between limits (as line segments)
    let steps = 8;
    let angle_range = max_rad - min_rad;
    if angle_range.abs() > 0.01 {
        let angle_step = angle_range / steps as f32;
        for i in 0..steps {
            let a1 = min_rad + angle_step * i as f32;
            let a2 = min_rad + angle_step * (i + 1) as f32;
            let p1 = center + Vec2::new(a1.cos(), a1.sin()) * arc_radius;
            let p2 = center + Vec2::new(a2.cos(), a2.sin()) * arc_radius;
            gizmos.line_2d(p1, p2, color);
        }
    }
}

/// Draw chain indicator (dashed line pattern showing chain direction)
fn draw_chain_indicator(
    gizmos: &mut Gizmos,
    start: Vec2,
    pos_offset: Vec2,
    length: usize,
    color: Color,
) {
    let max_preview = length.min(5); // Preview up to 5 chain links

    for i in 0..max_preview {
        let current = start + pos_offset * i as f32;
        let next = start + pos_offset * (i + 1) as f32;

        // Dashed line effect
        let direction = (next - current).normalize_or_zero();
        let total_length = (next - current).length();
        let dash_length = 2.0;
        let gap_length = 2.0;

        let mut pos = 0.0;
        while pos < total_length {
            let dash_start = current + direction * pos;
            let dash_end_raw = pos + dash_length;
            let dash_end = current + direction * dash_end_raw.min(total_length);
            gizmos.line_2d(dash_start, dash_end, color);
            pos += dash_length + gap_length;
        }
    }

    // If chain is longer than preview, show ellipsis indicator
    if length > max_preview {
        let end_pos = start + pos_offset * max_preview as f32;
        let dot_offset = pos_offset.normalize_or_zero() * 3.0;
        gizmos.circle_2d(end_pos + dot_offset, 1.0, color);
        gizmos.circle_2d(end_pos + dot_offset * 2.0, 1.0, color);
        gizmos.circle_2d(end_pos + dot_offset * 3.0, 1.0, color);
    }
}

/// Helper to parse Vec2 from TOML
fn parse_vec2(value: Option<&toml::Value>) -> Option<Vec2> {
    let arr = value?.as_array()?;
    if arr.len() < 2 {
        return None;
    }
    let x = arr[0]
        .as_float()
        .or_else(|| arr[0].as_integer().map(|i| i as f64))? as f32;
    let y = arr[1]
        .as_float()
        .or_else(|| arr[1].as_integer().map(|i| i as f64))? as f32;
    Some(Vec2::new(x, y))
}
