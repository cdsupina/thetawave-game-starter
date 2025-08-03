use avian2d::prelude::{
    AngleLimit, Collider, CollisionLayers, Friction, Joint, LockedAxes, Restitution, RevoluteJoint,
    RigidBody,
};
use bevy::{
    asset::Handle,
    ecs::{
        entity::Entity,
        error::{BevyError, Result},
        event::EventReader,
        resource::Resource,
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
    attributes::{JointedMob, MobAttributesComponent, MobAttributesResource, MobDecorationType},
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
            MobType::XhitaraTentacleShort | MobType::XhitaraTentacleLong => {
                self.xhitara_tentacle_start_mob_aseprite.clone()
            }
            MobType::XhitaraTentacleMiddle => self.xhitara_tentacle_middle_mob_aseprite.clone(),
            MobType::XhitaraTentacleEnd => self.xhitara_tentacle_end_mob_aseprite.clone(),
        }
    }

    fn get_mob_decoration(&self, mob_type: &MobDecorationType) -> Handle<Aseprite> {
        match mob_type {
            MobDecorationType::GruntThrusters => self.xhitara_grunt_thrusters_aseprite.clone(),
            MobDecorationType::ShooterThrusters => self.xhitara_spitter_thrusters_aseprite.clone(),
        }
    }
}

/// Used for the debug menu to disable behaviors and joints
/// Useful for aligning mob parts
#[derive(Resource)]
pub struct MobDebugSettings {
    pub joints_enabled: bool,
    pub behaviors_enabled: bool,
}

impl Default for MobDebugSettings {
    fn default() -> Self {
        Self {
            joints_enabled: true,
            behaviors_enabled: true,
        }
    }
}

/// Spawn a mob entity
pub(super) fn spawn_mob_system(
    mut cmds: Commands,
    assets: Res<GameAssets>,
    mob_debug_settings: Res<MobDebugSettings>,
    mut spawn_mob_event_reader: EventReader<SpawnMobEvent>,
    attributes_res: Res<MobAttributesResource>,
    behaviors_res: Res<MobBehaviorsResource>,
) -> Result {
    for event in spawn_mob_event_reader.read() {
        spawn_mob(
            &mut cmds,
            &event.mob_type,
            event.position,
            &mob_debug_settings,
            &attributes_res,
            &behaviors_res,
            &assets,
            false,
        )?;
    }
    Ok(())
}

/// Creates a revolute joint between two mob entities with optional angle limits
fn create_joint(
    cmds: &mut Commands,
    anchor: Entity,
    jointed: Entity,
    jointed_mob: &JointedMob,
    anchor_offset: Vec2,
) {
    // Create the revolute joint with anchor positions and compliance settings
    let mut joint = RevoluteJoint::new(anchor, jointed)
        .with_local_anchor_1(jointed_mob.anchor_1_pos + anchor_offset)
        .with_local_anchor_2(jointed_mob.anchor_2_pos)
        .with_compliance(jointed_mob.compliance);

    // Apply angle limits if specified (constrains how far the joint can rotate)
    if let Some(angle_limit_range) = &jointed_mob.angle_limit_range {
        joint.angle_limit = Some(AngleLimit::new(
            angle_limit_range.min.to_radians(),
            angle_limit_range.max.to_radians(),
        ));
        joint.angle_limit_torque = angle_limit_range.torque;
    }
    // Spawn the joint entity into the world
    cmds.spawn(joint);
}

/// Spawns a mob entity with all its components, decorations, and jointed sub-mobs
fn spawn_mob(
    cmds: &mut Commands,
    mob_type: &MobType,
    position: Vec2,
    mob_debug_settings: &MobDebugSettings,
    attributes_res: &MobAttributesResource,
    behaviors_res: &MobBehaviorsResource,
    assets: &GameAssets,
    suppress_jointed_mobs: bool,
) -> Result<Entity, BevyError> {
    info!("Spawning Mob: {:?} at {}", mob_type, position.to_string());

    // Look up the mob's configuration data from resources
    let mob_attributes = attributes_res
        .attributes
        .get(mob_type)
        .ok_or(BevyError::from("Mob attributes not found"))?;
    let mob_behavior_sequence = behaviors_res
        .behaviors
        .get(mob_type)
        .ok_or(BevyError::from("Mob behaviors not found"))?;

    // Spawn the main anchor entity with all core components
    let anchor_id = cmds
        .spawn((
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
            Friction::from(mob_attributes),
            Collider::from(mob_attributes),
            RigidBody::Dynamic,
            CollisionLayers::from(mob_attributes),
            LockedAxes::from(mob_attributes),
            Transform::from_xyz(position.x, position.y, mob_attributes.z_level),
            mob_behavior_sequence.clone().init_timer(),
        ))
        // Spawn visual decorations as child entities
        .with_children(|parent| {
            for (decoration_type, pos) in &mob_attributes.decorations {
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
        })
        .id();

    // Process all jointed sub-mobs (mobs connected via physics joints)
    for jointed_mob in &mob_attributes.jointed_mobs {
        // Handle chain spawning: creates a sequence of connected mobs
        if let Some(chain) = &jointed_mob.chain {
            let mut previous_id = anchor_id;
            let mut actual_length = chain.length;

            // Apply random chain termination logic if configured
            if let Some(random_chain) = &chain.random_chain {
                actual_length = random_chain.min_length;

                // Roll for early termination after minimum length is guaranteed
                for i in random_chain.min_length..chain.length {
                    if rand::random::<f32>() < random_chain.end_chance {
                        break;
                    }
                    actual_length = i + 1;
                }
            }

            // Spawn each mob in the chain and connect them with joints
            for chain_index in 0..actual_length {
                let jointed_id = spawn_mob(
                    cmds,
                    &jointed_mob.mob_type,
                    position + jointed_mob.offset_pos + chain.pos_offset * chain_index as f32,
                    mob_debug_settings,
                    attributes_res,
                    behaviors_res,
                    assets,
                    chain_index < actual_length - 1, // Suppress jointed mobs except on the last chain link
                )?;

                // Create joint between current and previous mob in chain
                // First link uses no anchor offset, subsequent links use chain.anchor_offset
                if mob_debug_settings.joints_enabled {
                    create_joint(
                        cmds,
                        previous_id,
                        jointed_id,
                        jointed_mob,
                        if chain_index != 0 {
                            chain.anchor_offset
                        } else {
                            Vec2::ZERO
                        },
                    );
                }
                // Update the previous_id for the next iteration
                previous_id = jointed_id;
            }
        } else if !suppress_jointed_mobs {
            // Handle single jointed mob (not part of a chain)
            let jointed_id = spawn_mob(
                cmds,
                &jointed_mob.mob_type,
                position + jointed_mob.offset_pos,
                mob_debug_settings,
                attributes_res,
                behaviors_res,
                assets,
                false,
            )?;
            // Connect the jointed mob directly to the anchor with no offset
            if mob_debug_settings.joints_enabled {
                create_joint(cmds, anchor_id, jointed_id, jointed_mob, Vec2::ZERO);
            }
        }
    }

    Ok(anchor_id)
}
