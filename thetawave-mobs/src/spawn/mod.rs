use avian2d::prelude::{
    AngleLimit, Collider, Joint, LockedAxes, Restitution, RevoluteJoint, RigidBody,
};
use bevy::{
    asset::Handle,
    ecs::{
        entity::Entity,
        error::{BevyError, Result},
        event::EventReader,
        system::{Commands, Res},
    },
    log::info,
    math::Vec2,
    prelude::Name,
    sprite::Sprite,
    transform::components::Transform,
};
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation, Aseprite};
use thetawave_assets::GameAssets;
use thetawave_states::{AppState, Cleanup};

use crate::{
    MobType, SpawnMobEvent,
    attributes::{MobAttributesComponent, MobAttributesResource, MobDecorationType},
    behavior::MobBehaviorsResource,
};

trait GameAssetsExt {
    fn get_mob_sprite(&self, mob_type: &MobType) -> Handle<Aseprite>;
    fn get_mob_decoration(&self, mob_type: &MobDecorationType) -> Handle<Aseprite>;
}

impl GameAssetsExt for GameAssets {
    fn get_mob_sprite(&self, mob_type: &MobType) -> Handle<Aseprite> {
        match mob_type {
            MobType::XhitaraGrunt => self.xhitara_grunt_mob_aseprite.clone(),
            MobType::XhitaraSpitter => self.xhitara_spitter_mob_aseprite.clone(),
            MobType::XhitaraGyro => self.xhitara_gyro_mob_aseprite.clone(),
            MobType::FreighterOne | MobType::FreighterTwo | MobType::FreighterFront => {
                self.freighter_front_mob_aseprite.clone()
            }
            MobType::FreighterMiddle => self.freighter_middle_mob_aseprite.clone(),
            MobType::FreighterBack => self.freighter_back_mob_aseprite.clone(),
            MobType::Trizetheron => self.trizetheron_mob_aseprite.clone(),
        }
    }

    fn get_mob_decoration(&self, mob_type: &MobDecorationType) -> Handle<Aseprite> {
        match mob_type {
            MobDecorationType::GruntThrusters => self.xhitara_grunt_thrusters_aseprite.clone(),
            MobDecorationType::ShooterThrusters => self.xhitara_spitter_thrusters_aseprite.clone(),
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
) -> Result {
    for event in spawn_mob_event_reader.read() {
        spawn_mob(
            &mut cmds,
            &event.mob_type,
            event.position,
            &attributes_res,
            &behaviors_res,
            &assets,
        )?;
    }
    Ok(())
}

fn spawn_mob(
    cmds: &mut Commands,
    mob_type: &MobType,
    position: Vec2,
    attributes_res: &MobAttributesResource,
    behaviors_res: &MobBehaviorsResource,
    assets: &GameAssets,
) -> Result<Entity, BevyError> {
    info!("Spawning Mob: {:?} at {}", mob_type, position.to_string());

    let mob_attributes = attributes_res
        .attributes
        .get(mob_type)
        .ok_or(BevyError::from("Mob attributes not found"))?;

    let mob_behavior_sequence = behaviors_res
        .behaviors
        .get(mob_type)
        .ok_or(BevyError::from("Mob behaviors not found"))?;

    let mut anchor_entity = cmds.spawn((
        Name::from(mob_attributes),
        MobAttributesComponent::from(mob_attributes),
        AseAnimation {
            animation: Animation::tag("idle"),
            aseprite: assets.get_mob_sprite(mob_type),
        },
        Sprite::default(),
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        Restitution::from(mob_attributes),
        Collider::from(mob_attributes),
        RigidBody::Dynamic,
        LockedAxes::from(mob_attributes),
        Transform::from_xyz(position.x, position.y, mob_attributes.z_level),
        mob_behavior_sequence.clone().init_timer(),
    ));

    let anchor_id = anchor_entity.id();

    // Spawn all decorations
    anchor_entity.with_children(|parent| {
        for (decoration_type, pos) in mob_attributes.decorations.iter() {
            parent.spawn((
                Transform::from_xyz(pos.x, pos.y, 0.0),
                AseAnimation {
                    animation: Animation::tag("idle"),
                    aseprite: assets.get_mob_decoration(decoration_type),
                },
                Sprite::default(),
                Name::new("Decoration"),
            ));
        }
    });

    for jointed_mob in mob_attributes.jointed_mobs.iter() {
        println!("{:?}", jointed_mob.mob_type);
        let jointed_id = spawn_mob(
            cmds,
            &jointed_mob.mob_type,
            position + jointed_mob.offset_pos,
            attributes_res,
            behaviors_res,
            assets,
        )?;

        let mut joint = RevoluteJoint::new(anchor_id, jointed_id)
            .with_local_anchor_1(jointed_mob.anchor_1_pos)
            .with_local_anchor_2(jointed_mob.anchor_2_pos)
            .with_compliance(jointed_mob.compliance);

        if let Some(angle_limit_range) = jointed_mob.angle_limit_range.as_ref() {
            joint.angle_limit = Some(AngleLimit::new(
                angle_limit_range.min.to_radians(),
                angle_limit_range.max.to_radians(),
            ));
            joint.angle_limit_torque = angle_limit_range.torque;
        }

        cmds.spawn(joint);
    }

    Ok(anchor_id)
}
