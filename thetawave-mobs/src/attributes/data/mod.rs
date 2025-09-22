use avian2d::prelude::{
    Collider, ColliderDensity, CollisionLayers, Friction, LockedAxes, PhysicsLayer, Restitution,
    Rotation,
};
use bevy::{
    ecs::{bundle::Bundle, component::Component, name::Name, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
    reflect::Reflect,
};
use serde::Deserialize;
use thetawave_core::HealthComponent;
use thetawave_physics::{ColliderShape, ThetawaveCollider, ThetawavePhysicsLayer};

mod joints;
mod spawners;

pub(crate) use joints::{JointedMob, JointsComponent};
pub(crate) use spawners::{MobSpawnerComponent, ProjectileSpawnerComponent};

const DEFAULT_COLLIDERS: &[ThetawaveCollider] = &[ThetawaveCollider {
    shape: ColliderShape::Rectangle(10.0, 10.0),
    position: Vec2::ZERO,
    rotation: 0.0,
}];
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
const DEFAULT_COLLISION_LAYER_MEMBERSHIP: &[ThetawavePhysicsLayer] =
    &[ThetawavePhysicsLayer::EnemyMob];
const DEFAULT_COLLISION_LAYER_FILTER: &[ThetawavePhysicsLayer] = &[
    ThetawavePhysicsLayer::AllyMob,
    ThetawavePhysicsLayer::AllyProjectile,
    ThetawavePhysicsLayer::EnemyMob,
    ThetawavePhysicsLayer::Player,
    ThetawavePhysicsLayer::EnemyTentacle,
];
const DEFAULT_COLLIDER_DENSITY: f32 = 1.0;
const DEFAULT_PROJECTILE_SPEED: f32 = 100.0;
const DEFAULT_PROJECTILE_DAMAGE: u32 = 5;
const DEFAULT_HEALTH: u32 = 50;
const DEFAULT_RANGE_SECONDS: f32 = 1.0;

/// Marker component for identifying mob entities in queries
#[derive(Component, Reflect, Debug, Clone)]
pub struct MobMarker {
    mob_type: String,
}

impl MobMarker {
    pub fn new(mob_type: impl Into<String>) -> Self {
        Self {
            mob_type: mob_type.into(),
        }
    }

    pub fn mob_type(&self) -> &str {
        &self.mob_type
    }
}

/// Mob attributes not directly used to make any other componnents
/// Typically used in mob behaviors
#[derive(Component, Reflect)]
pub(crate) struct MobAttributesComponent {
    pub linear_acceleration: Vec2,
    pub linear_deceleration: Vec2,
    pub max_linear_speed: Vec2,
    pub angular_acceleration: f32,
    pub angular_deceleration: f32,
    pub max_angular_speed: f32,
    pub targeting_range: Option<f32>,
    pub projectile_speed: f32,
    pub projectile_damage: u32,
    pub projectile_range_seconds: f32,
}

/// Contains all attributes for a mob
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobAttributes {
    #[serde(default = "default_colliders")]
    colliders: Vec<ThetawaveCollider>,
    name: String,
    #[serde(default = "default_z_level")]
    pub z_level: f32,
    #[serde(default = "default_rotation_locked")]
    rotation_locked: bool,
    #[serde(default = "default_max_linear_speed")]
    max_linear_speed: Vec2,
    #[serde(default = "default_linear_acceleration")]
    pub linear_acceleration: Vec2,
    #[serde(default = "default_linear_deceleration")]
    pub linear_deceleration: Vec2,
    #[serde(default = "default_angular_acceleration")]
    pub angular_acceleration: f32,
    #[serde(default = "default_angular_deceleration")]
    pub angular_deceleration: f32,
    #[serde(default = "default_max_angular_speed")]
    pub max_angular_speed: f32,
    #[serde(default = "default_restitution")]
    pub restitution: f32,
    #[serde(default = "default_friction")]
    pub friction: f32,
    #[serde(default)]
    pub decorations: Vec<(String, Vec2)>,
    #[serde(default)]
    pub sprite_key: Option<String>,
    #[serde(default)]
    pub jointed_mobs: Vec<JointedMob>,
    #[serde(default = "default_collision_layer_membership")]
    pub collision_layer_membership: Vec<ThetawavePhysicsLayer>,
    #[serde(default = "default_collision_layer_filter")]
    pub collision_layer_filter: Vec<ThetawavePhysicsLayer>,
    #[serde(default = "default_collider_density")]
    pub collider_density: f32,
    #[serde(default)]
    pub targeting_range: Option<f32>,
    #[serde(default)]
    pub mob_spawners: Option<MobSpawnerComponent>,
    #[serde(default)]
    pub projectile_spawners: Option<ProjectileSpawnerComponent>,
    #[serde(default = "default_projectile_speed")]
    pub projectile_speed: f32,
    #[serde(default = "default_projectile_damage")]
    pub projectile_damage: u32,
    #[serde(default)]
    pub behavior_transmitter: bool,
    #[serde(default = "default_health")]
    pub health: u32,
    #[serde(default = "default_range_seconds")]
    pub projectile_range_seconds: f32,
}

fn default_colliders() -> Vec<ThetawaveCollider> {
    DEFAULT_COLLIDERS.into()
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

fn default_restitution() -> f32 {
    DEFAULT_RESTITUTION
}

fn default_friction() -> f32 {
    DEFAULT_FRICTION
}

fn default_collision_layer_membership() -> Vec<ThetawavePhysicsLayer> {
    DEFAULT_COLLISION_LAYER_MEMBERSHIP.into()
}

fn default_collision_layer_filter() -> Vec<ThetawavePhysicsLayer> {
    DEFAULT_COLLISION_LAYER_FILTER.into()
}

fn default_collider_density() -> f32 {
    DEFAULT_COLLIDER_DENSITY
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

/// Resource for storing data for all mobs
/// Used mainly for spawning mobs with a given mob type string
#[derive(Deserialize, Debug, Resource)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobAttributesResource {
    pub attributes: HashMap<String, MobAttributes>,
}

/// Bundle containing all the core components needed for a mob entity
/// Simplifies mob spawning by grouping related components together
#[derive(Bundle)]
pub(crate) struct MobComponentBundle {
    pub name: Name,
    pub restitution: Restitution,
    pub friction: Friction,
    pub collision_layers: CollisionLayers,
    pub collider: Collider,
    pub locked_axes: LockedAxes,
    pub collider_density: ColliderDensity,
    pub mob_attributes: MobAttributesComponent,
    pub health: HealthComponent,
}

impl From<&MobAttributes> for MobComponentBundle {
    fn from(value: &MobAttributes) -> Self {
        // Calculate collision layers
        let mut membership: u32 = 0;
        for layer in &value.collision_layer_membership {
            membership |= layer.to_bits();
        }
        let mut filter: u32 = 0;
        for layer in &value.collision_layer_filter {
            filter |= layer.to_bits();
        }

        // Build compound collider
        let collider = Collider::compound(
            value
                .colliders
                .iter()
                .map(|c| {
                    (
                        c.position,
                        Rotation::degrees(c.rotation),
                        Collider::from(&c.shape),
                    )
                })
                .collect(),
        );

        // Determine locked axes
        let locked_axes = if value.rotation_locked {
            LockedAxes::ROTATION_LOCKED
        } else {
            LockedAxes::new()
        };

        MobComponentBundle {
            name: Name::new(value.name.clone()),
            restitution: Restitution::new(value.restitution),
            friction: Friction::new(value.friction),
            collision_layers: CollisionLayers::new(membership, filter),
            collider,
            locked_axes,
            collider_density: ColliderDensity(value.collider_density),
            mob_attributes: MobAttributesComponent {
                linear_acceleration: value.linear_acceleration,
                linear_deceleration: value.linear_deceleration,
                max_linear_speed: value.max_linear_speed,
                angular_acceleration: value.angular_acceleration,
                angular_deceleration: value.angular_deceleration,
                max_angular_speed: value.max_angular_speed,
                targeting_range: value.targeting_range,
                projectile_speed: value.projectile_speed,
                projectile_damage: value.projectile_damage,
                projectile_range_seconds: value.projectile_range_seconds,
            },
            health: HealthComponent::new(value.health),
        }
    }
}
