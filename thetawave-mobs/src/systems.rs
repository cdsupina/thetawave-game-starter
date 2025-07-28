use avian2d::prelude::{Collider, LockedAxes, RigidBody};
use bevy::{
    asset::Handle,
    ecs::{
        event::EventReader,
        system::{Commands, Res},
    },
    log::info,
    prelude::Name,
    sprite::Sprite,
    transform::components::Transform,
};
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation, Aseprite};
use thetawave_assets::GameAssets;
use thetawave_states::{AppState, Cleanup};

use crate::{data::MobAttributesResource, MobType, SpawnMobEvent};

trait GameAssetsExt {
    fn get_mob_sprite(&self, mob_type: &MobType) -> Handle<Aseprite>;
}

impl GameAssetsExt for GameAssets {
    fn get_mob_sprite(&self, mob_type: &MobType) -> Handle<Aseprite> {
        match mob_type {
            MobType::Grunt => self.grunt_mob_aseprite.clone(),
            MobType::Shooter => self.shooter_mob_aseprite.clone(),
        }
    }
}

/// Spawn a mob entity
pub(super) fn spawn_mob_system(
    mut cmds: Commands,
    assets: Res<GameAssets>,
    mut spawn_mob_event_reader: EventReader<SpawnMobEvent>,
    mob_resource: Res<MobAttributesResource>,
) -> std::result::Result<(), bevy::prelude::BevyError> {
    for event in spawn_mob_event_reader.read() {
        info!(
            "Spawning Mob: {:?} at {}",
            event.mob_type,
            event.position.to_string()
        );

        let mob_attributes = mob_resource
            .attributes
            .get(&event.mob_type)
            .ok_or(bevy::prelude::BevyError::from("Mob attributes not found"))?;

        cmds.spawn((
            Name::from(mob_attributes),
            AseAnimation {
                animation: Animation::tag("idle"),
                aseprite: assets.get_mob_sprite(&event.mob_type),
            },
            Sprite::default(),
            Cleanup::<AppState> {
                states: vec![AppState::Game],
            },
            Collider::from(mob_attributes),
            RigidBody::Dynamic,
            LockedAxes::from(mob_attributes),
            Transform::from_xyz(
                event.position.x,
                event.position.y,
                mob_attributes.get_z_level(),
            ),
        ));
    }

    Ok(())
}
