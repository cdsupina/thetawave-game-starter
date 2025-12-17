use bevy::{
    ecs::{component::Component, entity::Entity, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
};

/// Represents a connected group of mobs (via physics joints)
#[derive(Debug, Clone)]
pub struct MobGroup {
    /// All entities in this group
    pub members: Vec<Entity>,
    /// Cached center position (updated each frame)
    pub center_position: Vec2,
}

/// Resource tracking all mob groups and selection state
#[derive(Resource, Default)]
pub struct MobGroupRegistry {
    /// All current mob groups, keyed by a representative entity from the group
    pub groups: HashMap<Entity, MobGroup>,
    /// Currently selected group's key entity
    pub selected_group: Option<Entity>,
    /// Index for Tab cycling through groups
    pub selection_index: usize,
    /// List of group key entities for consistent ordering during Tab cycling
    pub group_order: Vec<Entity>,
}

impl MobGroupRegistry {
    /// Cycle to next mob group, returns the newly selected group
    pub fn cycle_next(&mut self) -> Option<Entity> {
        if self.group_order.is_empty() {
            self.selected_group = None;
            return None;
        }

        self.selection_index = (self.selection_index + 1) % self.group_order.len();
        self.selected_group = Some(self.group_order[self.selection_index]);
        self.selected_group
    }

    /// Rebuild group order list from current groups
    pub fn rebuild_order(&mut self) {
        self.group_order = self.groups.keys().cloned().collect();
        self.group_order.sort_by_key(|e| e.index());

        // Fix selection if current selection was removed
        if let Some(selected) = self.selected_group {
            if !self.groups.contains_key(&selected) {
                self.selected_group = self.group_order.first().cloned();
                self.selection_index = 0;
            }
        } else if !self.group_order.is_empty() {
            // Auto-select first group if nothing selected
            self.selected_group = self.group_order.first().cloned();
            self.selection_index = 0;
        }
    }
}

/// Collected stats for display in the debug UI
#[derive(Debug, Clone)]
pub struct MobDisplayStats {
    pub entity: Entity,
    pub mob_type: String,
    pub name: String,

    // Health
    pub current_health: u32,
    pub max_health: u32,

    // Movement
    pub linear_velocity: Vec2,
    pub angular_velocity: f32,
    pub max_linear_speed: Vec2,
    pub linear_acceleration: Vec2,
    pub max_angular_speed: f32,
    pub angular_acceleration: f32,

    // Physics
    pub position: Vec2,
    pub rotation: f32,
    pub restitution: f32,
    pub friction: f32,

    // Combat (if has projectile spawners)
    pub projectile_speed: f32,
    pub projectile_damage: u32,
    pub targeting_range: Option<f32>,

    // Behavior
    pub is_behavior_transmitter: bool,
    pub behavior_receiver_from: Option<Entity>,
}

/// Stats for an entire mob group
#[derive(Debug, Clone, Default, Resource)]
pub struct MobGroupDisplayStats {
    pub group_center: Vec2,
    pub total_health: u32,
    pub max_total_health: u32,
    pub member_stats: Vec<MobDisplayStats>,
}

/// Behavior tree display data for a mob
#[derive(Debug, Clone)]
pub struct BehaviorTreeDisplay {
    pub mob_entity: Entity,
    pub mob_type: String,
    /// Whether this mob has its own behavior tree
    pub has_own_tree: bool,
    /// Entity this mob receives behaviors from (if any)
    pub receives_from: Option<Entity>,
    /// Name of the currently active action node (e.g., "Phase 1 - Center Position")
    pub active_node_name: Option<String>,
    /// Currently active behavior types (e.g., "MoveDown", "MoveTo { x: 0.0, y: 50.0 }")
    pub active_actions: Vec<String>,
}

/// Resource for storing behavior tree displays
#[derive(Debug, Clone, Default, Resource)]
pub struct BehaviorTreeDisplays {
    pub displays: Vec<BehaviorTreeDisplay>,
}

/// Marker component for the Mob View window
#[derive(Component)]
pub struct MobViewWindow;

/// Marker component for the Mob View camera
#[derive(Component)]
pub struct MobViewCamera;

/// Resource tracking the mob view window state
#[derive(Resource)]
pub struct MobViewWindowState {
    pub window_entity: Option<Entity>,
    pub camera_entity: Option<Entity>,
    pub is_open: bool,
    pub zoom_level: f32,
}

impl Default for MobViewWindowState {
    fn default() -> Self {
        Self {
            window_entity: None,
            camera_entity: None,
            is_open: false,
            zoom_level: 1.0, // Camera scale (0.5 = zoomed in, 2.0 = zoomed out)
        }
    }
}
