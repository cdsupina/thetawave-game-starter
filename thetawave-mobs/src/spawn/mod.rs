use avian2d::prelude::{
    AngleLimit, Collider, ColliderDensity, CollisionLayers, Friction, Joint, LockedAxes,
    Restitution, RevoluteJoint, RigidBody,
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
    math::{Quat, Vec2},
    platform::collections::HashMap,
    prelude::Name,
    sprite::Sprite,
    transform::components::Transform,
};
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation, Aseprite};
use bevy_behave::prelude::BehaveTree;
use thetawave_assets::{GameAssets, ParticleMaterials};
use thetawave_core::HealthComponent;
use thetawave_particles::{ParticleEffectType, spawn_particle_effect};
use thetawave_projectiles::ProjectileType;
use thetawave_states::{AppState, Cleanup};

use crate::{
    MobType, SpawnMobEvent,
    attributes::{
        JointedMob, JointsComponent, MobAttributesComponent, MobAttributesResource,
        MobDecorationType,
    },
    behavior::{BehaviorReceiverComponent, MobBehaviorsResource},
};

trait GameAssetsExt {
    fn get_mob_sprite(&self, mob_type: &MobType) -> Handle<Aseprite>;
    fn get_mob_decoration_sprite(&self, mob_type: &MobDecorationType) -> Handle<Aseprite>;
}

impl GameAssetsExt for GameAssets {
    /// Get the Aseprite handle from from a given MobType
    fn get_mob_sprite(&self, mob_type: &MobType) -> Handle<Aseprite> {
        match mob_type {
            MobType::XhitaraGrunt => self.xhitara_grunt_mob_aseprite.clone(),
            MobType::XhitaraSpitter => self.xhitara_spitter_mob_aseprite.clone(),
            MobType::XhitaraGyro => self.xhitara_gyro_mob_aseprite.clone(),
            MobType::FreighterOne | MobType::FreighterTwo => {
                self.freighter_front_mob_aseprite.clone()
            }
            MobType::FreighterMiddle => self.freighter_middle_mob_aseprite.clone(),
            MobType::FreighterBack => self.freighter_back_mob_aseprite.clone(),
            MobType::Trizetheron => self.trizetheron_mob_aseprite.clone(),
            MobType::TrizetheronLeftHead => self.trizetheron_left_head_mob_aseprite.clone(),
            MobType::TrizetheronRightHead => self.trizetheron_right_head_mob_aseprite.clone(),
            MobType::XhitaraTentacleShort | MobType::XhitaraTentacleLong => {
                self.xhitara_tentacle_start_mob_aseprite.clone()
            }
            MobType::XhitaraTentacleMiddle => self.xhitara_tentacle_middle_mob_aseprite.clone(),
            MobType::XhitaraTentacleEnd => self.xhitara_tentacle_end_mob_aseprite.clone(),
            MobType::XhitaraCyclusk => self.xhitara_cyclusk_mob_aseprite.clone(),
            MobType::XhitaraPacer => self.xhitara_pacer_mob_aseprite.clone(),
            MobType::XhitaraMissile => self.xhitara_missile_mob_aseprite.clone(),
            MobType::XhitaraLauncher => self.xhitara_launcher_mob_aseprite.clone(),
            MobType::Ferritharax => self.ferritharax_head_mob_aseprite.clone(),
            MobType::FerritharaxBody => self.ferritharax_body_mob_aseprite.clone(),
            MobType::FerritharaxRightShoulder => {
                self.ferritharax_right_shoulder_mob_aseprite.clone()
            }
            MobType::FerritharaxLeftShoulder => self.ferritharax_left_shoulder_mob_aseprite.clone(),
            MobType::FerritharaxRightClaw => self.ferritharax_right_claw_mob_aseprite.clone(),
            MobType::FerritharaxLeftClaw => self.ferritharax_left_claw_mob_aseprite.clone(),
            MobType::FerritharaxLeftArm => self.ferritharax_left_arm_mob_aseprite.clone(),
            MobType::FerritharaxRightArm => self.ferritharax_right_arm_mob_aseprite.clone(),
        }
    }

    /// Get the Aseprite handle for a decoration using a given MobDecorationType
    fn get_mob_decoration_sprite(&self, mob_type: &MobDecorationType) -> Handle<Aseprite> {
        match mob_type {
            MobDecorationType::XhitaraGruntThrusters => {
                self.xhitara_grunt_thrusters_aseprite.clone()
            }
            MobDecorationType::XhitaraSpitterThrusters => {
                self.xhitara_spitter_thrusters_aseprite.clone()
            }
            MobDecorationType::XhitaraPacerThrusters => {
                self.xhitara_pacer_thrusters_aseprite.clone()
            }
            MobDecorationType::XhitaraMissileThrusters => {
                self.xhitara_missile_thrusters_aseprite.clone()
            }
            MobDecorationType::FreighterThrusters => self.freighter_thrusters_aseprite.clone(),
            MobDecorationType::XhitaraLauncherThrusters => {
                self.xhitara_launcher_thrusters_aseprite.clone()
            }
        }
    }
}

trait ParticleEffectTypeExt {
    fn from_projectile_type(projectile_type: &ProjectileType) -> ParticleEffectType;
}

impl ParticleEffectTypeExt for ParticleEffectType {
    fn from_projectile_type(projectile_type: &ProjectileType) -> ParticleEffectType {
        match projectile_type {
            ProjectileType::Bullet => todo!(),
            ProjectileType::Blast => Self::SpawnBlast,
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

/// Reads SpawnMobEvents and spawns mobs
pub(super) fn spawn_mob_system(
    mut cmds: Commands,
    assets: Res<GameAssets>,
    materials: Res<ParticleMaterials>,
    mob_debug_settings: Res<MobDebugSettings>,
    mut spawn_mob_event_reader: EventReader<SpawnMobEvent>,
    attributes_res: Res<MobAttributesResource>,
    behaviors_res: Res<MobBehaviorsResource>,
) -> Result {
    for event in spawn_mob_event_reader.read() {
        let suppress_jointed_mobs = false;
        let transmitter_entity: Option<Entity> = None;

        spawn_mob(
            &mut cmds,
            &event.mob_type,
            event.position,
            event.rotation,
            &mob_debug_settings,
            &attributes_res,
            &behaviors_res,
            &assets,
            &materials,
            suppress_jointed_mobs,
            transmitter_entity,
        )?;
    }
    Ok(())
}

/// Spawns a mob entity with all its components, decorations, and jointed sub-mobs
fn spawn_mob(
    cmds: &mut Commands,
    mob_type: &MobType,
    position: Vec2,
    rotation: f32,
    mob_debug_settings: &MobDebugSettings,
    attributes_res: &MobAttributesResource,
    behaviors_res: &MobBehaviorsResource,
    assets: &GameAssets,
    materials: &ParticleMaterials,
    suppress_jointed_mobs: bool,
    transmitter_entity: Option<Entity>, // entity that can transmit behaviors to the mob
) -> Result<Entity, BevyError> {
    info!("Spawning Mob: {:?} at {}", mob_type, position.to_string());

    // Look up the mob's configuration data from resources
    let mob_attributes = attributes_res
        .attributes
        .get(mob_type)
        .ok_or(BevyError::from("Mob attributes not found"))?;
    // Spawn the main anchor entity with all core components
    let mut entity_commands = cmds.spawn((
        Name::from(mob_attributes),
        mob_type.clone(),
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
        ColliderDensity::from(mob_attributes),
        RigidBody::Dynamic,
        CollisionLayers::from(mob_attributes),
        LockedAxes::from(mob_attributes),
        Transform::from_xyz(position.x, position.y, mob_attributes.z_level)
            .with_rotation(Quat::from_rotation_z(rotation.to_radians())),
        HealthComponent::from(mob_attributes),
    ));

    if let Some(mob_spawners) = &mob_attributes.mob_spawners {
        entity_commands.insert(mob_spawners.clone());
    }

    if let Some(entity) = transmitter_entity {
        entity_commands.insert(BehaviorReceiverComponent(entity));
    }

    let anchor_id = entity_commands
        .with_children(|parent| {
            // Spawn visual decorations as child entities
            for (decoration_type, pos) in &mob_attributes.decorations {
                parent.spawn((
                    Transform::from_xyz(pos.x, pos.y, 0.0),
                    AseAnimation {
                        animation: Animation::tag("idle"),
                        aseprite: assets.get_mob_decoration_sprite(decoration_type),
                    },
                    Sprite::default(),
                    Name::new("Decoration"),
                ));
            }

            // Spawn behavior tree
            if let Some(tree) = behaviors_res.behaviors.get(mob_type) {
                parent.spawn((
                    Name::new("Mob Behavior Tree"),
                    BehaveTree::new(tree.clone()).with_logging(true),
                ));
            }
        })
        .id();

    // Set the transmitter entity for the spawned joints
    let new_transmitter_entity = if mob_attributes.behavior_transmitter {
        Some(anchor_id)
    } else {
        transmitter_entity
    };

    let mut mob_joints = HashMap::new();

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
                    0.0,
                    mob_debug_settings,
                    attributes_res,
                    behaviors_res,
                    assets,
                    materials,
                    chain_index < actual_length - 1, // Suppress jointed mobs except on the last chain link
                    new_transmitter_entity,
                )?;

                // Create joint between current and previous mob in chain
                // First link uses no anchor offset, subsequent links use chain.anchor_offset
                if mob_debug_settings.joints_enabled {
                    mob_joints.insert(
                        jointed_mob.key.clone(),
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
                        ),
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
                0.0,
                mob_debug_settings,
                attributes_res,
                behaviors_res,
                assets,
                materials,
                false,
                new_transmitter_entity,
            )?;
            // Connect the jointed mob directly to the anchor with no offset
            if mob_debug_settings.joints_enabled {
                mob_joints.insert(
                    jointed_mob.key.clone(),
                    create_joint(cmds, anchor_id, jointed_id, jointed_mob, Vec2::ZERO),
                );
            }
        }
    }

    // Add joints component to the anchor entity if we have any joints
    if !mob_joints.is_empty() {
        cmds.entity(anchor_id)
            .insert(JointsComponent { joints: mob_joints });
    }

    // Now spawn particle effects and update projectile spawners
    if let Some(ref mut projectile_spawners) = mob_attributes.projectile_spawners.clone() {
        for (_, spawner) in projectile_spawners.spawners.iter_mut() {
            // Spawn particle effect directly and store the entity reference
            let transform = Transform::from_translation(spawner.position.extend(0.0));
            let particle_entity = spawn_particle_effect(
                cmds,
                Some(anchor_id),
                &ParticleEffectType::from_projectile_type(&spawner.projectile_type),
                &spawner.faction,
                &transform,
                assets,
                materials,
            );

            spawner.spawn_effect_entity = Some(particle_entity);
        }

        // Update the entity with the modified projectile spawners
        cmds.entity(anchor_id).insert(projectile_spawners.clone());
    }

    Ok(anchor_id)
}

/// Creates a revolute joint between two mob entities with optional angle limits
fn create_joint(
    cmds: &mut Commands,
    anchor: Entity,
    jointed: Entity,
    jointed_mob: &JointedMob,
    anchor_offset: Vec2,
) -> Entity {
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
    cmds.spawn(joint).id()
}
