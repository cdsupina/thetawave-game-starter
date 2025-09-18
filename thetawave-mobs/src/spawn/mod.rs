use avian2d::prelude::{AngleLimit, Joint, RevoluteJoint, RigidBody};
use bevy::{
    asset::Handle,
    ecs::{
        entity::Entity,
        error::{BevyError, Result},
        event::{Event, EventReader, EventWriter},
        query::With,
        resource::Resource,
        system::{Commands, Query, Res},
    },
    log::warn,
    math::{Quat, Vec2},
    platform::collections::HashMap,
    prelude::Name,
    sprite::Sprite,
    transform::components::Transform,
};
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation, Aseprite};
use bevy_behave::prelude::BehaveTree;
use thetawave_assets::{AssetError, AssetResolver, ExtendedGameAssets, GameAssets};
use thetawave_core::{AppState, Cleanup};
use thetawave_particles::{SpawnParticleEffectEvent, SpawnerParticleEffectSpawnedEvent};
use thetawave_projectiles::ProjectileType;

use crate::{
    MobMarker,
    attributes::{
        JointedMob, JointsComponent, MobAttributesResource, MobComponentBundle,
        ProjectileSpawnerComponent,
    },
    behavior::{BehaviorReceiverComponent, MobBehaviorsResource},
};

/// Get the Aseprite handle from a decoration name string using asset resolver
fn get_mob_decoration_sprite(
    decoration_name: &str,
    extended_assets: &ExtendedGameAssets,
    game_assets: &GameAssets,
) -> Result<Handle<Aseprite>, AssetError> {
    AssetResolver::get_game_sprite(decoration_name, extended_assets, game_assets)
}

fn get_particle_effect_str(projectile_type: &ProjectileType) -> &str {
    match projectile_type {
        ProjectileType::Bullet => "spawn_bullet",
        ProjectileType::Blast => "spawn_blast",
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

/// Event for spawning mobs using a mob type string and position
#[derive(Event, Debug)]
pub struct SpawnMobEvent {
    pub mob_type: String,
    pub position: Vec2,
    pub rotation: f32,
}

impl SpawnMobEvent {
    pub fn new(mob_type: impl Into<String>, position: Vec2, rotation: f32) -> Self {
        Self {
            mob_type: mob_type.into(),
            position,
            rotation,
        }
    }
}

/// Reads SpawnMobEvents and spawns mobs
pub(super) fn spawn_mob_system(
    mut cmds: Commands,
    game_assets: Res<GameAssets>,
    extended_assets: Res<ExtendedGameAssets>,
    mob_debug_settings: Res<MobDebugSettings>,
    mut spawn_mob_event_reader: EventReader<SpawnMobEvent>,
    attributes_res: Res<MobAttributesResource>,
    behaviors_res: Res<MobBehaviorsResource>,
    mut particle_effect_event_writer: EventWriter<SpawnParticleEffectEvent>,
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
            &game_assets,
            &extended_assets,
            suppress_jointed_mobs,
            transmitter_entity,
            &mut particle_effect_event_writer,
        )?;
    }
    Ok(())
}

/// Spawns a mob entity with all its components, decorations, and jointed sub-mobs
fn spawn_mob(
    cmds: &mut Commands,
    mob_type: &str,
    position: Vec2,
    rotation: f32,
    mob_debug_settings: &MobDebugSettings,
    attributes_res: &MobAttributesResource,
    behaviors_res: &MobBehaviorsResource,
    game_assets: &GameAssets,
    extended_assets: &ExtendedGameAssets,
    suppress_jointed_mobs: bool,
    transmitter_entity: Option<Entity>, // entity that can transmit behaviors to the mob
    particle_effect_event_writer: &mut EventWriter<SpawnParticleEffectEvent>,
) -> Result<Entity, BevyError> {
    // Look up the mob's configuration data from resources
    let mob_attributes = attributes_res
        .attributes
        .get(mob_type)
        .ok_or(BevyError::from("Mob attributes not found"))?;
    // Spawn the main anchor entity with all core components
    let mut entity_commands = cmds.spawn((
        MobComponentBundle::from(mob_attributes),
        MobMarker::new(mob_type),
        AseAnimation {
            animation: Animation::tag("idle"),
            aseprite: {
                let sprite_key = if let Some(sprite_key) = &mob_attributes.sprite_key {
                    sprite_key.as_str()
                } else {
                    mob_type
                };
                AssetResolver::get_game_sprite(sprite_key, extended_assets, game_assets)?
            },
        },
        Sprite::default(),
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        RigidBody::Dynamic,
        Transform::from_xyz(position.x, position.y, mob_attributes.z_level)
            .with_rotation(Quat::from_rotation_z(rotation.to_radians())),
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
            for (decoration_sprite_stem, pos) in &mob_attributes.decorations {
                parent.spawn((
                    Transform::from_xyz(pos.x, pos.y, 0.0),
                    AseAnimation {
                        animation: Animation::tag("idle"),
                        aseprite: match get_mob_decoration_sprite(
                            decoration_sprite_stem,
                            extended_assets,
                            game_assets,
                        ) {
                            Ok(handle) => handle,
                            Err(e) => {
                                warn!(
                                    "Failed to load decoration sprite, skipping decoration: {}",
                                    e
                                );
                                continue;
                            }
                        },
                    },
                    Sprite::default(),
                    Name::new(decoration_sprite_stem.clone()),
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
                    game_assets,
                    extended_assets,
                    chain_index < actual_length - 1, // Suppress jointed mobs except on the last chain link
                    new_transmitter_entity,
                    particle_effect_event_writer,
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
                game_assets,
                extended_assets,
                false,
                new_transmitter_entity,
                particle_effect_event_writer,
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
        for (key, spawner) in projectile_spawners.spawners.iter_mut() {
            // Spawn particle effect directly and store the entity reference
            let transform = Transform::from_translation(spawner.position.extend(0.0));

            particle_effect_event_writer.write(SpawnParticleEffectEvent {
                parent_entity: Some(anchor_id),
                effect_type: get_particle_effect_str(&spawner.projectile_type).to_string(),
                faction: spawner.faction.clone(),
                transform,
                is_active: false,
                key: Some(key.to_string()),
                needs_position_tracking: false, // Spawner effects use parent-child relationship
                is_one_shot: false,
            });
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

pub(crate) fn connect_effect_to_spawner(
    mut events: EventReader<SpawnerParticleEffectSpawnedEvent>,
    mut mob_query: Query<&mut ProjectileSpawnerComponent, With<MobMarker>>,
) {
    for event in events.read() {
        // Directly access the parent mob using the parent_entity from the event
        if let Ok(mut projectile_spawner_component) = mob_query.get_mut(event.parent_entity) {
            // Check if this mob has a spawner with the matching key
            if let Some(spawner) = projectile_spawner_component.spawners.get_mut(&event.key) {
                spawner.spawn_effect_entity = Some(event.effect_entity);
            } else {
                warn!(
                    "Mob {} has no spawner with key '{}'",
                    event.parent_entity.index(),
                    event.key
                );
            }
        } else {
            warn!(
                "Parent entity {} is not a valid mob with ProjectileSpawnerComponent",
                event.parent_entity.index()
            );
        }
    }
}
