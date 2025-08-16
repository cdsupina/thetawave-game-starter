use bevy::{ecs::event::Event, math::Vec2};
use thetawave_core::Faction;

#[derive(Debug)]
pub enum ProjectileType {
    Bullet,
    Blast,
}

#[derive(Event)]
pub struct SpawnProjectileEvent {
    pub projectile_type: ProjectileType,
    pub faction: Faction,
    pub position: Vec2,
    pub rotation: f32,
}
