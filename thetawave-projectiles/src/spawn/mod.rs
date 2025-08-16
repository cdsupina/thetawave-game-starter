use avian2d::prelude::Collider;
use bevy::{
    asset::Handle,
    color::Color,
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
use thetawave_core::Faction;
use thetawave_states::{AppState, Cleanup};

use crate::{ProjectileType, SpawnProjectileEvent, attributes::ProjectileAttributesResource};

trait GameAssetsExt {
    fn get_projecitle_sprite(&self, projectile_type: &ProjectileType) -> Handle<Aseprite>;
}

impl GameAssetsExt for GameAssets {
    fn get_projecitle_sprite(&self, projectile_type: &ProjectileType) -> Handle<Aseprite> {
        match projectile_type {
            ProjectileType::Bullet => self.bullet_projectile_aseprite.clone(),
            ProjectileType::Blast => self.blast_projectile_aseprite.clone(),
        }
    }
}

trait FactionExt {
    fn get_projectile_color(&self, projectile_type: &ProjectileType) -> Color;
}

impl FactionExt for Faction {
    fn get_projectile_color(&self, projectile_type: &ProjectileType) -> Color {
        match projectile_type {
            ProjectileType::Bullet => match self {
                Faction::Ally => Color::srgba(0.0, 0.0, 5.0, 1.0),
                Faction::Enemy => Color::srgba(5.0, 0.0, 0.0, 1.0),
            },
            ProjectileType::Blast => match self {
                Faction::Ally => Color::srgba(5.0, 5.0, 0.0, 0.5),
                Faction::Enemy => Color::srgba(5.0, 0.0, 0.0, 0.5),
            },
        }
    }
}

pub(super) fn spawn_projectile_system(
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
            event.rotation,
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
    rotation: f32,
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

    let entity = cmds
        .spawn((
            Name::new("Projectile"),
            Sprite {
                color: faction.get_projectile_color(projectile_type),
                ..Default::default()
            },
            Collider::from(projectile_attributes),
            AseAnimation {
                animation: Animation::tag("idle"),
                aseprite: assets.get_projecitle_sprite(projectile_type),
            },
            Cleanup::<AppState> {
                states: vec![AppState::Game],
            },
            Transform::from_xyz(position.x, position.y, 0.0)
                .with_rotation(Quat::from_rotation_z(rotation.to_radians())),
        ))
        .id();

    Ok(entity)
}
