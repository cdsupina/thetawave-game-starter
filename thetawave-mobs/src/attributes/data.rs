use avian2d::prelude::{
    Collider, ColliderDensity, CollisionLayers, Friction, LockedAxes, PhysicsLayer, Restitution,
    Rotation,
};
use bevy::{
    ecs::{component::Component, event::Event, name::Name, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
};
use serde::Deserialize;
use strum_macros::EnumIter;
use thetawave_physics::ThetawavePhysicsLayer;

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
const DEFAULT_MAX_ANGULAR_SPEED: f32 = 1.0;
const DEFAULT_RESTITUTION: f32 = 0.5;
const DEFAULT_FRICTION: f32 = 0.5;
const DEFAULT_COLLISION_LAYER_MEMBERSHIP: &[ThetawavePhysicsLayer] =
    &[ThetawavePhysicsLayer::Enemy];
const DEFAULT_COLLISION_LAYER_FILTER: &[ThetawavePhysicsLayer] = &[
    ThetawavePhysicsLayer::Ally,
    ThetawavePhysicsLayer::Enemy,
    ThetawavePhysicsLayer::Player,
    ThetawavePhysicsLayer::Tentacle,
];
const DEFAULT_COLLIDER_DENSITY: f32 = 1.0;

/// Describes a collider that can be attached to mobs
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct ThetawaveCollider {
    pub shape: ColliderShape,
    pub position: Vec2,
    pub rotation: f32,
}

/// All types of collider shapes that can be attached to mobs
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) enum ColliderShape {
    Circle(f32),
    Rectangle(f32, f32),
}

impl From<&ColliderShape> for Collider {
    fn from(value: &ColliderShape) -> Self {
        match value {
            ColliderShape::Circle(radius) => Collider::circle(*radius),
            ColliderShape::Rectangle(width, height) => Collider::rectangle(*width, *height),
        }
    }
}

// All types of decorations that can be attached to mobs
#[derive(Deserialize, Debug, Clone)]
pub(crate) enum MobDecorationType {
    XhitaraGruntThrusters,
    XhitaraSpitterThrusters,
    XhitaraPacerThrusters,
    XhitaraMissileThrusters,
    FreighterThrusters,
}

/// All types of spawnable mobs
#[derive(Deserialize, Debug, Eq, PartialEq, Hash, EnumIter, Clone)]
pub enum MobType {
    XhitaraGrunt,
    XhitaraSpitter,
    XhitaraGyro,
    FreighterOne,
    FreighterTwo,
    FreighterFront,
    FreighterMiddle,
    FreighterBack,
    XhitaraCyclusk,
    Trizetheron,
    XhitaraTentacleShort,
    XhitaraTentacleLong,
    XhitaraTentacleMiddle,
    XhitaraTentacleEnd,
    XhitaraPacer,
    XhitaraMissile,
}

/// Event for spawning mobs using a mob type and position
#[derive(Event, Debug)]
pub struct SpawnMobEvent {
    pub mob_type: MobType,
    pub position: Vec2,
}

/// Component to hold mob attributes that are not used in cases outside of creating components
/// Such as in mob behaviors
#[derive(Component)]
pub(crate) struct MobAttributesComponent {
    pub linear_acceleration: Vec2,
    pub linear_deceleration: Vec2,
    pub max_linear_speed: Vec2,
    pub angular_acceleration: f32,
    pub max_angular_speed: f32,
}

/// Describes an angle limit for a joint
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct JointAngleLimit {
    pub min: f32,
    pub max: f32,
    pub torque: f32,
}

/// Used for making chains of random length
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RandomMobChain {
    pub min_length: u8,
    pub end_chance: f32,
}

/// Describes a chain of mobs that are spawned and jointed together
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobChain {
    pub length: u8,
    pub pos_offset: Vec2,
    pub anchor_offset: Vec2,
    pub random_chain: Option<RandomMobChain>,
}

/// Mob that is also spawned and jointed to the original mob
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct JointedMob {
    pub mob_type: MobType,
    #[serde(default)]
    pub offset_pos: Vec2,
    #[serde(default)]
    pub anchor_1_pos: Vec2,
    #[serde(default)]
    pub anchor_2_pos: Vec2,
    #[serde(default)]
    pub angle_limit_range: Option<JointAngleLimit>,
    #[serde(default)]
    pub compliance: f32,
    #[serde(default)]
    pub chain: Option<MobChain>,
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
    #[serde(default = "default_max_angular_speed")]
    pub max_angular_speed: f32,
    #[serde(default = "default_restitution")]
    pub restitution: f32,
    #[serde(default = "default_friction")]
    pub friction: f32,
    #[serde(default)]
    pub decorations: Vec<(MobDecorationType, Vec2)>,
    #[serde(default)]
    pub jointed_mobs: Vec<JointedMob>,
    #[serde(default = "default_collision_layer_membership")]
    pub collision_layer_membership: Vec<ThetawavePhysicsLayer>,
    #[serde(default = "default_collision_layer_filter")]
    pub collision_layer_filter: Vec<ThetawavePhysicsLayer>,
    #[serde(default = "default_collider_density")]
    pub collider_density: f32,
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

fn default_max_angular_speed() -> f32 {
    DEFAULT_MAX_ANGULAR_SPEED
}

/// Resource tracking all data for mobs
#[derive(Deserialize, Debug, Resource)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobAttributesResource {
    pub attributes: HashMap<MobType, MobAttributes>,
}

impl From<&MobAttributes> for Restitution {
    fn from(value: &MobAttributes) -> Self {
        Restitution::new(value.restitution)
    }
}

impl From<&MobAttributes> for Friction {
    fn from(value: &MobAttributes) -> Self {
        Friction::new(value.friction)
    }
}

impl From<&MobAttributes> for CollisionLayers {
    fn from(value: &MobAttributes) -> Self {
        let mut membership: u32 = 0;

        for layer in &value.collision_layer_membership {
            membership |= layer.to_bits();
        }

        let mut filter: u32 = 0;

        for layer in &value.collision_layer_filter {
            filter |= layer.to_bits();
        }

        CollisionLayers::new(membership, filter)
    }
}

/// Create a collider component using mob attributes
impl From<&MobAttributes> for Collider {
    fn from(value: &MobAttributes) -> Self {
        Collider::compound(
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
        )
    }
}

impl From<&MobAttributes> for Name {
    fn from(value: &MobAttributes) -> Self {
        Name::new(value.name.clone())
    }
}

impl From<&MobAttributes> for LockedAxes {
    fn from(value: &MobAttributes) -> Self {
        let rotation_locked = value.rotation_locked;

        if rotation_locked {
            return LockedAxes::ROTATION_LOCKED;
        }

        // unlock rotation if rotation locked is not true
        LockedAxes::new()
    }
}

impl From<&MobAttributes> for ColliderDensity {
    fn from(value: &MobAttributes) -> Self {
        ColliderDensity(value.collider_density)
    }
}

impl From<&MobAttributes> for MobAttributesComponent {
    fn from(value: &MobAttributes) -> Self {
        MobAttributesComponent {
            linear_acceleration: value.linear_acceleration,
            linear_deceleration: value.linear_deceleration,
            max_linear_speed: value.max_linear_speed,
            angular_acceleration: value.angular_acceleration,
            max_angular_speed: value.max_angular_speed,
        }
    }
}
