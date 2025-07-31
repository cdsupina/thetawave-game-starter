use avian2d::prelude::{Collider, LockedAxes, Restitution, RigidBody};
use bevy::{
    asset::Handle,
    ecs::{
        error::BevyError,
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

use crate::{
    MobType, SpawnMobEvent,
    attributes::{MobAttributesComponent, MobAttributesResource},
    behavior::MobBehaviorsResource,
};

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
    attributes_res: Res<MobAttributesResource>,
    behaviors_res: Res<MobBehaviorsResource>,
) -> std::result::Result<(), bevy::prelude::BevyError> {
    for event in spawn_mob_event_reader.read() {
        info!(
            "Spawning Mob: {:?} at {}",
            event.mob_type,
            event.position.to_string()
        );

        let mob_attributes = attributes_res
            .attributes
            .get(&event.mob_type)
            .ok_or(BevyError::from("Mob attributes not found"))?;

        let mob_behavior_sequence = behaviors_res
            .behaviors
            .get(&event.mob_type)
            .ok_or(BevyError::from("Mob behaviors not found"))?;

        cmds.spawn((
            Name::from(mob_attributes),
            MobAttributesComponent::from(mob_attributes),
            AseAnimation {
                animation: Animation::tag("idle"),
                aseprite: assets.get_mob_sprite(&event.mob_type),
            },
            Sprite::default(),
            Cleanup::<AppState> {
                states: vec![AppState::Game],
            },
            Restitution::from(mob_attributes),
            Collider::from(mob_attributes),
            RigidBody::Dynamic,
            LockedAxes::from(mob_attributes),
            Transform::from_xyz(event.position.x, event.position.y, mob_attributes.z_level),
            mob_behavior_sequence.clone().init_timer(),
        ));
    }

    Ok(())
}
