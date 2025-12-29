//! MobAsset type definition for .mob files.

use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
use thetawave_physics::{ThetawaveCollider, ThetawavePhysicsLayer};

use crate::{
    asset::MobRef,
    attributes::{MobSpawnerComponent, ProjectileSpawnerComponent},
    behavior::BehaviorNodeData,
};

// Default constants (matching existing MobAttributes defaults)
const DEFAULT_Z_LEVEL: f32 = 0.0;
const DEFAULT_ROTATION_LOCKED: bool = true;
const DEFAULT_MAX_LINEAR_SPEED: Vec2 = Vec2::new(20.0, 20.0);
const DEFAULT_LINEAR_ACCELERATION: Vec2 = Vec2::new(0.1, 0.1);
const DEFAULT_LINEAR_DECELERATION: Vec2 = Vec2::new(0.3, 0.3);
const DEFAULT_ANGULAR_ACCELERATION: f32 = 0.1;
const DEFAULT_ANGULAR_DECELERATION: f32 = 0.1;
const DEFAULT_MAX_ANGULAR_SPEED: f32 = 1.0;
const DEFAULT_RESTITUTION: f32 = 0.5;
const DEFAULT_FRICTION: f32 = 0.5;
const DEFAULT_COLLIDER_DENSITY: f32 = 1.0;
const DEFAULT_PROJECTILE_SPEED: f32 = 100.0;
const DEFAULT_PROJECTILE_DAMAGE: u32 = 5;
const DEFAULT_HEALTH: u32 = 50;
const DEFAULT_RANGE_SECONDS: f32 = 1.0;
const DEFAULT_SPAWNABLE: bool = true;

/// A single mob definition loaded from a .mob file.
///
/// This combines attributes and behavior into a single struct that is
/// deserialized from raw TOML values after optional patch merging.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MobAsset {
    /// Display name for the mob
    pub name: String,

    /// Whether this mob can be spawned directly (appears in debug spawn menu)
    /// Set to false for jointed parts that should only spawn as part of a parent mob
    #[serde(default = "default_spawnable")]
    pub spawnable: bool,

    // === Physical attributes ===
    /// Collision shapes for the mob
    #[serde(default = "default_colliders")]
    pub colliders: Vec<ThetawaveCollider>,

    /// Z-level for rendering order
    #[serde(default = "default_z_level")]
    pub z_level: f32,

    /// Whether rotation is locked
    #[serde(default = "default_rotation_locked")]
    pub rotation_locked: bool,

    // === Movement attributes ===
    /// Maximum linear speed [x, y]
    #[serde(default = "default_max_linear_speed")]
    pub max_linear_speed: Vec2,

    /// Linear acceleration [x, y]
    #[serde(default = "default_linear_acceleration")]
    pub linear_acceleration: Vec2,

    /// Linear deceleration [x, y]
    #[serde(default = "default_linear_deceleration")]
    pub linear_deceleration: Vec2,

    /// Angular acceleration (radians/s^2)
    #[serde(default = "default_angular_acceleration")]
    pub angular_acceleration: f32,

    /// Angular deceleration (radians/s^2)
    #[serde(default = "default_angular_deceleration")]
    pub angular_deceleration: f32,

    /// Maximum angular speed (radians/s)
    #[serde(default = "default_max_angular_speed")]
    pub max_angular_speed: f32,

    // === Physics ===
    /// Bounciness (0.0 = no bounce, 1.0 = full bounce)
    #[serde(default = "default_restitution")]
    pub restitution: f32,

    /// Friction coefficient
    #[serde(default = "default_friction")]
    pub friction: f32,

    /// Density for physics calculations
    #[serde(default = "default_collider_density")]
    pub collider_density: f32,

    /// Collision layer membership
    #[serde(default = "default_collision_layer_membership")]
    pub collision_layer_membership: Vec<ThetawavePhysicsLayer>,

    /// Collision layer filter (what this mob collides with)
    #[serde(default = "default_collision_layer_filter")]
    pub collision_layer_filter: Vec<ThetawavePhysicsLayer>,

    // === Combat ===
    /// Health points
    #[serde(default = "default_health")]
    pub health: u32,

    /// Range at which this mob can target
    #[serde(default)]
    pub targeting_range: Option<f32>,

    /// Speed of spawned projectiles
    #[serde(default = "default_projectile_speed")]
    pub projectile_speed: f32,

    /// Damage of spawned projectiles
    #[serde(default = "default_projectile_damage")]
    pub projectile_damage: u32,

    /// How long projectiles last (seconds)
    #[serde(default = "default_range_seconds")]
    pub projectile_range_seconds: f32,

    // === Visual ===
    /// Sprite file path relative to assets directory.
    /// Example: "media/aseprite/xhitara_grunt_mob.aseprite"
    pub sprite: String,

    /// Decorative sprites attached to this mob
    #[serde(default)]
    pub decorations: Vec<(String, Vec2)>,

    // === Spawners ===
    /// Mob spawner configuration
    #[serde(default)]
    pub mob_spawners: Option<MobSpawnerComponent>,

    /// Projectile spawner configuration
    #[serde(default)]
    pub projectile_spawners: Option<ProjectileSpawnerComponent>,

    // === Joints ===
    /// Mobs that are jointed to this mob
    #[serde(default)]
    pub jointed_mobs: Vec<JointedMobRef>,

    // === Behavior ===
    /// Whether this mob can transmit behaviors to children
    #[serde(default)]
    pub behavior_transmitter: bool,

    /// Inline behavior tree definition
    #[serde(default)]
    pub behavior: Option<BehaviorNodeData>,
}

/// Reference to a jointed mob using a file path instead of a string key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointedMobRef {
    /// Identifier key for this joint (used by behaviors)
    pub key: String,

    /// Path to the .mob file, e.g., "mobs/ferritharax/body.mob"
    /// Automatically normalized to "ferritharax/body" format.
    pub mob_ref: MobRef,

    /// Position offset from parent mob
    #[serde(default)]
    pub offset_pos: Vec2,

    /// Anchor point on the parent mob
    #[serde(default)]
    pub anchor_1_pos: Vec2,

    /// Anchor point on the child mob
    #[serde(default)]
    pub anchor_2_pos: Vec2,

    /// Angle limits for the joint
    #[serde(default)]
    pub angle_limit_range: Option<JointAngleLimit>,

    /// Joint compliance (flexibility, lower = stiffer)
    #[serde(default)]
    pub compliance: f32,

    /// Chain configuration for creating linked sequences
    #[serde(default)]
    pub chain: Option<MobChain>,
}

/// Describes angle limits for a revolute joint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointAngleLimit {
    /// Minimum angle (degrees)
    pub min: f32,
    /// Maximum angle (degrees)
    pub max: f32,
    /// Resistance torque
    pub torque: f32,
}

/// Configuration for creating a chain of jointed mobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MobChain {
    /// Number of links in the chain
    pub length: u8,
    /// Position offset between chain links
    pub pos_offset: Vec2,
    /// Anchor offset between chain links
    pub anchor_offset: Vec2,
    /// Random chain termination settings
    #[serde(default)]
    pub random_chain: Option<RandomMobChain>,
}

/// Settings for random chain length termination.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RandomMobChain {
    /// Minimum guaranteed chain length
    pub min_length: u8,
    /// Probability that each additional link ends the chain
    pub end_chance: f32,
}

// Default value functions

fn default_colliders() -> Vec<ThetawaveCollider> {
    vec![ThetawaveCollider {
        shape: thetawave_physics::ColliderShape::Rectangle(10.0, 10.0),
        position: Vec2::ZERO,
        rotation: 0.0,
    }]
}

fn default_z_level() -> f32 {
    DEFAULT_Z_LEVEL
}

fn default_rotation_locked() -> bool {
    DEFAULT_ROTATION_LOCKED
}

fn default_max_linear_speed() -> Vec2 {
    DEFAULT_MAX_LINEAR_SPEED
}

fn default_linear_acceleration() -> Vec2 {
    DEFAULT_LINEAR_ACCELERATION
}

fn default_linear_deceleration() -> Vec2 {
    DEFAULT_LINEAR_DECELERATION
}

fn default_angular_acceleration() -> f32 {
    DEFAULT_ANGULAR_ACCELERATION
}

fn default_angular_deceleration() -> f32 {
    DEFAULT_ANGULAR_DECELERATION
}

fn default_max_angular_speed() -> f32 {
    DEFAULT_MAX_ANGULAR_SPEED
}

fn default_restitution() -> f32 {
    DEFAULT_RESTITUTION
}

fn default_friction() -> f32 {
    DEFAULT_FRICTION
}

fn default_collider_density() -> f32 {
    DEFAULT_COLLIDER_DENSITY
}

fn default_collision_layer_membership() -> Vec<ThetawavePhysicsLayer> {
    vec![ThetawavePhysicsLayer::EnemyMob]
}

fn default_collision_layer_filter() -> Vec<ThetawavePhysicsLayer> {
    vec![
        ThetawavePhysicsLayer::AllyMob,
        ThetawavePhysicsLayer::AllyProjectile,
        ThetawavePhysicsLayer::EnemyMob,
        ThetawavePhysicsLayer::Player,
        ThetawavePhysicsLayer::EnemyTentacle,
    ]
}

fn default_projectile_speed() -> f32 {
    DEFAULT_PROJECTILE_SPEED
}

fn default_projectile_damage() -> u32 {
    DEFAULT_PROJECTILE_DAMAGE
}

fn default_health() -> u32 {
    DEFAULT_HEALTH
}

fn default_range_seconds() -> f32 {
    DEFAULT_RANGE_SECONDS
}

fn default_spawnable() -> bool {
    DEFAULT_SPAWNABLE
}
