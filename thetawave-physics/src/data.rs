use avian2d::prelude::{Collider, PhysicsLayer};
#[cfg(feature = "physics_debug")]
use bevy::ecs::resource::Resource;
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

#[cfg(feature = "physics_debug")]
#[derive(Resource, Default)]
pub struct PhysicsDebugSettings {
    pub gizmos_enabled: bool,
    pub diagnostics_enabled: bool,
}

#[derive(PhysicsLayer, Default, Serialize, Deserialize, Debug, Clone)]
pub enum ThetawavePhysicsLayer {
    #[default]
    Default,
    EnemyMob,
    AllyMob,
    Player,
    EnemyProjectile,
    AllyProjectile,
    EnemyTentacle,
}

/// Describes a collider that can be attached to mobs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThetawaveCollider {
    pub shape: ColliderShape,
    pub position: Vec2,
    pub rotation: f32,
}

/// All types of collider shapes that can be attached to mobs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ColliderShape {
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
