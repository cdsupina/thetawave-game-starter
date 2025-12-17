use bevy::ecs::{query::Without, system::{Res, Single}};
use bevy_behave::{ego_tree, Behave};
use bevy_egui::{
    EguiContext, PrimaryEguiContext,
    egui::{self, Color32, ScrollArea, SidePanel, TopBottomPanel, CollapsingHeader, ProgressBar},
};
use thetawave_mobs::MobBehaviorsResource;

use super::data::{
    BehaviorTreeDisplay, BehaviorTreeDisplays, MobDisplayStats, MobGroupDisplayStats,
    MobGroupRegistry,
};

/// System that renders the mob view UI in the secondary window
/// Runs in MobViewContextPass schedule which provides the correct egui context
pub fn mob_view_ui_system(
    egui_context: Single<&mut EguiContext, Without<PrimaryEguiContext>>,
    registry: Res<MobGroupRegistry>,
    group_stats: Res<MobGroupDisplayStats>,
    tree_displays: Res<BehaviorTreeDisplays>,
    behaviors_resource: Res<MobBehaviorsResource>,
) {
    let mut binding = egui_context.into_inner();
    let ctx = binding.get_mut();

    // Left panel - Stats
    SidePanel::left("mob_stats_panel")
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Mob Group Stats");
            ui.separator();

            // Group summary
            if registry.selected_group.is_some() {
                let member_count = group_stats.member_stats.len();
                if member_count > 1 {
                    ui.label(format!("Multipart Mob ({} parts)", member_count));
                } else {
                    ui.label("Single Mob");
                }
                ui.label(format!(
                    "Center: ({:.1}, {:.1})",
                    group_stats.group_center.x, group_stats.group_center.y
                ));

                ui.add_space(8.0);

                // Total health bar
                ui.label("Total Health:");
                let health_ratio = group_stats.total_health as f32
                    / group_stats.max_total_health.max(1) as f32;
                ui.add(
                    ProgressBar::new(health_ratio)
                        .text(format!(
                            "{}/{}",
                            group_stats.total_health, group_stats.max_total_health
                        ))
                        .fill(health_color(health_ratio)),
                );

                ui.separator();

                // Per-member stats in collapsible sections
                ScrollArea::vertical().show(ui, |ui| {
                    for stats in &group_stats.member_stats {
                        render_mob_stats_section(ui, stats);
                    }
                });
            } else {
                ui.label("No mob selected");
                ui.add_space(8.0);
                ui.label("Press Tab to cycle through mobs");
            }
        });

    // Right panel - Behavior Trees
    SidePanel::right("behavior_panel")
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Behavior Trees");
            ui.separator();

            // Only show mobs that have behavior trees or receive behaviors
            let relevant_displays: Vec<_> = tree_displays
                .displays
                .iter()
                .filter(|d| d.has_own_tree || d.receives_from.is_some())
                .collect();

            if relevant_displays.is_empty() {
                ui.label("No behavior trees active");
            } else {
                ScrollArea::vertical().show(ui, |ui| {
                    for tree_display in relevant_displays {
                        render_behavior_tree_section(ui, tree_display, &behaviors_resource);
                    }
                });
            }
        });

    // Top panel - Controls (above the camera view)
    TopBottomPanel::top("controls_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Controls:");
            ui.separator();
            ui.label("Tab - Cycle mobs");
            ui.separator();
            ui.label("+/- - Zoom");
        });
    });

    // No CentralPanel - let the camera view show through
}

fn health_color(ratio: f32) -> Color32 {
    if ratio > 0.6 {
        Color32::from_rgb(50, 200, 50)
    } else if ratio > 0.3 {
        Color32::from_rgb(200, 200, 50)
    } else {
        Color32::from_rgb(200, 50, 50)
    }
}

fn render_mob_stats_section(ui: &mut egui::Ui, stats: &MobDisplayStats) {
    CollapsingHeader::new(&stats.name)
        .id_salt(stats.entity.index()) // Ensure unique ID for mobs with same name
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Entity:");
                ui.label(format!("{}", stats.entity.index()));
            });
            ui.horizontal(|ui| {
                ui.label("Type:");
                ui.label(&stats.mob_type);
            });

            // Health
            ui.group(|ui| {
                ui.label("Health");
                let ratio = stats.current_health as f32 / stats.max_health.max(1) as f32;
                ui.add(
                    ProgressBar::new(ratio)
                        .text(format!("{}/{}", stats.current_health, stats.max_health))
                        .fill(health_color(ratio)),
                );
            });

            // Movement
            ui.group(|ui| {
                ui.label("Movement");
                ui.horizontal(|ui| {
                    ui.label("Velocity:");
                    ui.label(format!(
                        "({:.1}, {:.1})",
                        stats.linear_velocity.x, stats.linear_velocity.y
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Angular Vel:");
                    ui.label(format!("{:.2} rad/s", stats.angular_velocity));
                });
                ui.horizontal(|ui| {
                    ui.label("Max Speed:");
                    ui.label(format!(
                        "({:.1}, {:.1})",
                        stats.max_linear_speed.x, stats.max_linear_speed.y
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Acceleration:");
                    ui.label(format!(
                        "({:.2}, {:.2})",
                        stats.linear_acceleration.x, stats.linear_acceleration.y
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Max Angular:");
                    ui.label(format!("{:.2} rad/s", stats.max_angular_speed));
                });
                ui.horizontal(|ui| {
                    ui.label("Angular Accel:");
                    ui.label(format!("{:.2}", stats.angular_acceleration));
                });
            });

            // Physics
            ui.group(|ui| {
                ui.label("Physics");
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.label(format!("({:.1}, {:.1})", stats.position.x, stats.position.y));
                });
                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    ui.label(format!("{:.1}\u{00B0}", stats.rotation));
                });
                ui.horizontal(|ui| {
                    ui.label("Restitution:");
                    ui.label(format!("{:.2}", stats.restitution));
                });
                ui.horizontal(|ui| {
                    ui.label("Friction:");
                    ui.label(format!("{:.2}", stats.friction));
                });
            });

            // Combat
            ui.group(|ui| {
                ui.label("Combat");
                ui.horizontal(|ui| {
                    ui.label("Proj Speed:");
                    ui.label(format!("{:.1}", stats.projectile_speed));
                });
                ui.horizontal(|ui| {
                    ui.label("Proj Damage:");
                    ui.label(format!("{}", stats.projectile_damage));
                });
                if let Some(range) = stats.targeting_range {
                    ui.horizontal(|ui| {
                        ui.label("Target Range:");
                        ui.label(format!("{:.1}", range));
                    });
                }
            });

            // Behavior info - only show if there's relevant behavior data
            if stats.is_behavior_transmitter || stats.behavior_receiver_from.is_some() {
                ui.group(|ui| {
                    ui.label("Behavior");
                    if stats.is_behavior_transmitter {
                        ui.colored_label(Color32::from_rgb(100, 200, 255), "Has joint connections");
                    }
                    if let Some(receiver_from) = stats.behavior_receiver_from {
                        ui.label(format!("Receives from: Entity {}", receiver_from.index()));
                    }
                });
            }
        });
}

fn render_behavior_tree_section(
    ui: &mut egui::Ui,
    display: &BehaviorTreeDisplay,
    behaviors_resource: &MobBehaviorsResource,
) {
    let header_text = format!(
        "{} (Entity {})",
        display.mob_type,
        display.mob_entity.index()
    );

    CollapsingHeader::new(header_text)
        .default_open(display.has_own_tree) // Auto-expand mobs with their own trees
        .show(ui, |ui| {
            // Show relationship info for behavior receivers
            if let Some(receiver_from) = display.receives_from {
                ui.colored_label(
                    Color32::from_rgb(200, 200, 100),
                    format!("Receives behaviors from Entity {}", receiver_from.index()),
                );
            }

            // Look up the tree definition and render structure (only for mobs with their own tree)
            if display.has_own_tree {
                if let Some(tree) = behaviors_resource.behaviors.get(&display.mob_type) {
                    ui.add_space(4.0);
                    ui.label("Tree Structure:");
                    render_tree_node(ui, tree.root(), 0, display.active_node_name.as_deref());
                } else {
                    ui.colored_label(Color32::GRAY, "Tree definition not found");
                }
            }

            // Show currently running behaviors
            ui.add_space(8.0);
            ui.label("Running Behaviors:");
            if display.active_actions.is_empty() {
                ui.colored_label(Color32::GRAY, "  (none)");
            } else {
                for action in &display.active_actions {
                    ui.colored_label(Color32::from_rgb(100, 255, 100), format!("  • {}", action));
                }
            }
        });
}

/// Renders a behavior tree node and its children recursively
fn render_tree_node(
    ui: &mut egui::Ui,
    node: ego_tree::NodeRef<Behave>,
    depth: usize,
    active_node_name: Option<&str>,
) {
    let indent = "  ".repeat(depth);
    let node_name = get_behave_node_name(node.value());

    // Determine if this node should be highlighted
    let is_active = match node.value() {
        Behave::DynamicEntity { name, .. } => {
            // Action node is active only if its name matches the active node
            active_node_name.is_some_and(|active| active == name.as_ref())
        }
        Behave::Forever | Behave::Sequence | Behave::Fallback | Behave::While => {
            // Control nodes are active when any action is running (tree is executing)
            active_node_name.is_some()
        }
        _ => false,
    };

    // Render the node
    let color = if is_active {
        Color32::from_rgb(100, 255, 100) // Green for active
    } else {
        Color32::from_rgb(180, 180, 180) // Gray for inactive
    };

    ui.colored_label(color, format!("{}{}", indent, node_name));

    // Render children
    for child in node.children() {
        render_tree_node(ui, child, depth + 1, active_node_name);
    }
}

/// Gets a display name for a Behave node
fn get_behave_node_name(behave: &Behave) -> String {
    match behave {
        Behave::Wait(seconds) => format!("Wait({:.1}s)", seconds),
        Behave::DynamicEntity { name, .. } => format!("▶ {}", name),
        Behave::Sequence => "Sequence".to_string(),
        Behave::Fallback => "Fallback".to_string(),
        Behave::Invert => "Invert".to_string(),
        Behave::AlwaysSucceed => "AlwaysSucceed".to_string(),
        Behave::AlwaysFail => "AlwaysFail".to_string(),
        Behave::TriggerReq(_) => "TriggerReq".to_string(),
        Behave::Forever => "Forever ↻".to_string(),
        Behave::While => "While".to_string(),
        Behave::IfThen => "IfThen".to_string(),
    }
}
