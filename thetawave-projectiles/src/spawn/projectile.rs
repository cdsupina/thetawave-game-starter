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
    math::{Quat, Vec2},
    sprite::Sprite,
    transform::components::Transform,
};
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation, Aseprite};
use thetawave_assets::{AssetResolver, ExtendedGameAssets, GameAssets};
use thetawave_core::{CollisionDamage, Faction};
use thetawave_states::{AppState, Cleanup};

use crate::{
    ProjectileType, SpawnProjectileEvent,
    attributes::{ProjectileAttributesResource, ProjectileRangeComponent},
    spawn::FactionExt,
};

/// Get the Aseprite handle from a given ProjectileType using asset resolver
fn get_projectile_sprite(
    projectile_type: &ProjectileType,
    extended_assets: &ExtendedGameAssets,
    game_assets: &GameAssets,
) -> Handle<Aseprite> {
    let key = match projectile_type {
        ProjectileType::Bullet => "bullet_projectile",
        ProjectileType::Blast => "blast_projectile",
    };

    AssetResolver::get_game_sprite(key, extended_assets, game_assets)
}

pub(crate) fn spawn_projectile_system(
    mut cmds: Commands,
    game_assets: Res<GameAssets>,
    extended_assets: Res<ExtendedGameAssets>,
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
            &game_assets,
            &extended_assets,
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
    game_assets: &GameAssets,
    extended_assets: &ExtendedGameAssets,
    attributes_res: &ProjectileAttributesResource,
) -> Result<Entity, BevyError> {
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
            aseprite: get_projectile_sprite(projectile_type, extended_assets, game_assets),
        },
        RigidBody::Dynamic,
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        Transform::from_xyz(position.x, position.y, 0.0)
            .with_rotation(Quat::from_rotation_z(rotation)),
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
