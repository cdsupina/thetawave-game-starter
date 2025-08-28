use avian2d::prelude::{Collider, CollisionEventsEnabled, LinearVelocity, RigidBody, Sensor};
use bevy::{
    asset::Handle,
    ecs::{
        entity::Entity,
        error::{BevyError, Result},
        event::EventReader,
        name::Name,
        system::{Commands, Res},
    },
    log::info,
    math::{Quat, Vec2},
    sprite::Sprite,
    transform::components::Transform,
};
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation, Aseprite};
use thetawave_assets::GameAssets;
use thetawave_core::{CollisionDamage, Faction};
use thetawave_states::{AppState, Cleanup};

use crate::{
    ProjectileType, SpawnProjectileEvent,
    attributes::{ProjectileAttributesResource, ProjectileRangeComponent},
    spawn::FactionExt,
};

trait GameAssetsExt {
    fn get_projectile_sprite(&self, projectile_type: &ProjectileType) -> Handle<Aseprite>;
}

impl GameAssetsExt for GameAssets {
    fn get_projectile_sprite(&self, projectile_type: &ProjectileType) -> Handle<Aseprite> {
        match projectile_type {
            ProjectileType::Bullet => self.bullet_projectile_aseprite.clone(),
            ProjectileType::Blast => self.blast_projectile_aseprite.clone(),
        }
    }
}

pub(crate) fn spawn_projectile_system(
    mut cmds: Commands,
    assets: Res<GameAssets>,
    mut spawn_projectile_event_reader: EventReader<SpawnProjectileEvent>,
    attributes_res: Res<ProjectileAttributesResource>,
) -> Result {
    for event in spawn_projectile_event_reader.read() {
        spawn_projectile(
            &mut cmds,
            &event.projectile_type,
            &event.faction,
            event.position,
            event.velocity,
            event.damage,
            event.range_seconds,
            &assets,
            &attributes_res,
        )?;
    }

    Ok(())
}

fn spawn_projectile(
    cmds: &mut Commands,
    projectile_type: &ProjectileType,
    faction: &Faction,
    position: Vec2,
    velocity: Vec2,
    damage: u32,
    range_seconds: f32,
    assets: &GameAssets,
    attributes_res: &ProjectileAttributesResource,
) -> Result<Entity, BevyError> {
    info!(
        "Spawning Projectile: {:?} at {}",
        projectile_type,
        position.to_string()
    );

    // Look up the projectiles's configuration data from resources
    let projectile_attributes = attributes_res
        .attributes
        .get(projectile_type)
        .ok_or(BevyError::from("Projectile attributes not found"))?;

    // Calculate the projectile's rotation from its velocity vector
    let rotation = velocity.y.atan2(velocity.x);

    let mut entity_cmds = cmds.spawn((
        Name::new("Projectile"),
        projectile_type.clone(),
        faction.clone(),
        Sprite {
            color: faction.get_projectile_color(projectile_type),
            ..Default::default()
        },
        Collider::from(projectile_attributes),
        AseAnimation {
            animation: Animation::tag("idle"),
            aseprite: assets.get_projectile_sprite(projectile_type),
        },
        RigidBody::Dynamic,
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        Transform::from_xyz(position.x, position.y, 0.0)
            .with_rotation(Quat::from_rotation_z(rotation.to_radians())),
        LinearVelocity(velocity),
        CollisionEventsEnabled,
        CollisionDamage(damage),
        ProjectileRangeComponent::new(range_seconds),
    ));

    if projectile_attributes.is_sensor {
        entity_cmds.insert(Sensor);
    }

    Ok(entity_cmds.id())
}
