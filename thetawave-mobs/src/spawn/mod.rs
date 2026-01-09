use avian2d::prelude::{
    AngleLimit, Collider, ColliderDensity, CollisionLayers, Friction, LockedAxes, PhysicsLayer,
    Restitution, RevoluteJoint, RigidBody, Rotation,
};
use bevy::{
    asset::Handle,
    ecs::{
        entity::Entity,
        error::{BevyError, Result},
        message::{Message, MessageReader, MessageWriter},
        query::With,
        resource::Resource,
        system::{Commands, Query, Res},
    },
    math::{Quat, Vec2},
    platform::collections::HashMap,
    prelude::Name,
    sprite::Sprite,
    transform::components::Transform,
};
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation, Aseprite};
use bevy_behave::prelude::BehaveTree;
use thetawave_assets::{AssetError, AssetResolver, MergedGameAssets};
#[cfg(feature = "debug")]
use thetawave_core::LoggingSettings;
use thetawave_core::{AppState, Cleanup, HealthComponent};
use thetawave_particles::{SpawnSpawnerEffectEvent, SpawnerParticleEffectSpawnedEvent};
use thetawave_projectiles::ProjectileType;

use bevy::ecs::bundle::Bundle;

use crate::{
    MobMarker,
    asset::{JointedMobRef, MobAsset, MobRef, MobRegistry},
    attributes::{JointsComponent, MobAttributesComponent, ProjectileSpawnerComponent},
    behavior::BehaviorReceiverComponent,
};

/// Bundle containing all the core physics and gameplay components for a mob entity.
/// Note: Collider is added separately to support mobs with no colliders.
#[derive(Bundle)]
struct MobComponentBundle {
    name: Name,
    restitution: Restitution,
    friction: Friction,
    locked_axes: LockedAxes,
    mob_attributes: MobAttributesComponent,
    health: HealthComponent,
}

/// Bundle for collider-related components (only added when mob has colliders)
#[derive(Bundle)]
struct MobColliderBundle {
    collision_layers: CollisionLayers,
    collider: Collider,
    collider_density: ColliderDensity,
}

impl MobColliderBundle {
    /// Create collider bundle from mob asset, returns None if mob has no colliders
    fn from_mob(mob: &MobAsset) -> Option<Self> {
        if mob.colliders.is_empty() {
            return None;
        }

        // Calculate collision layers
        let mut membership: u32 = 0;
        for layer in &mob.collision_layer_membership {
            membership |= layer.to_bits();
        }
        let mut filter: u32 = 0;
        for layer in &mob.collision_layer_filter {
            filter |= layer.to_bits();
        }

        // Build compound collider
        let collider = Collider::compound(
            mob.colliders
                .iter()
                .map(|c| {
                    (
                        c.position,
                        Rotation::degrees(c.rotation),
                        Collider::from(&c.shape),
                    )
                })
                .collect(),
        );

        Some(MobColliderBundle {
            collision_layers: CollisionLayers::new(membership, filter),
            collider,
            collider_density: ColliderDensity(mob.collider_density),
        })
    }
}

impl From<&MobAsset> for MobComponentBundle {
    fn from(mob: &MobAsset) -> Self {
        // Determine locked axes
        let locked_axes = if mob.rotation_locked {
            LockedAxes::ROTATION_LOCKED
        } else {
            LockedAxes::new()
        };

        MobComponentBundle {
            name: Name::new(mob.name.clone()),
            restitution: Restitution::new(mob.restitution),
            friction: Friction::new(mob.friction),
            locked_axes,
            mob_attributes: MobAttributesComponent {
                linear_acceleration: mob.linear_acceleration,
                linear_deceleration: mob.linear_deceleration,
                max_linear_speed: mob.max_linear_speed,
                angular_acceleration: mob.angular_acceleration,
                angular_deceleration: mob.angular_deceleration,
                max_angular_speed: mob.max_angular_speed,
                targeting_range: mob.targeting_range,
                projectile_speed: mob.projectile_speed,
                projectile_damage: mob.projectile_damage,
                projectile_range_seconds: mob.projectile_range_seconds,
            },
            health: HealthComponent::new(mob.health),
        }
    }
}

/// Strip the game:// prefix from a path if present
fn strip_extended_prefix(path: &str) -> &str {
    path.strip_prefix("game://").unwrap_or(path)
}

/// Extract asset key from sprite path.
/// "media/aseprite/xhitara_grunt_mob.aseprite" → "xhitara_grunt_mob"
/// "game://media/aseprite/foo.aseprite" → "foo"
fn sprite_path_to_key(path: &str) -> &str {
    let path = strip_extended_prefix(path);
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
}

/// Get the Aseprite handle from a sprite path using asset resolver.
/// Supports game:// prefix to hint at extended asset location.
fn get_sprite_from_path(
    sprite_path: &str,
    game_assets: &MergedGameAssets,
) -> Result<Handle<Aseprite>, AssetError> {
    let key = sprite_path_to_key(sprite_path);
    AssetResolver::get_game_sprite(key, game_assets)
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

/// Event for spawning mobs using a mob reference path and position.
///
/// The `mob_ref` can be specified in two formats:
/// - Full path: "mobs/ferritharax/head.mob"
/// - Normalized key: "ferritharax/head"
///
/// Both formats are automatically normalized to the key format.
#[derive(Message, Debug)]
pub struct SpawnMobEvent {
    pub mob_ref: MobRef,
    pub position: Vec2,
    pub rotation: f32,
}

impl SpawnMobEvent {
    pub fn new(mob_ref: impl Into<MobRef>, position: Vec2, rotation: f32) -> Self {
        Self {
            mob_ref: mob_ref.into(),
            position,
            rotation,
        }
    }
}

/// Reads SpawnMobEvents and spawns mobs
pub(super) fn spawn_mob_system(
    mut cmds: Commands,
    game_assets: Res<MergedGameAssets>,
    mob_debug_settings: Res<MobDebugSettings>,
    #[cfg(feature = "debug")] logging_settings: Res<LoggingSettings>,
    mut spawn_mob_event_reader: MessageReader<SpawnMobEvent>,
    mob_registry: Res<MobRegistry>,
    mut spawner_effect_event_writer: MessageWriter<SpawnSpawnerEffectEvent>,
) -> Result {
    for event in spawn_mob_event_reader.read() {
        let suppress_jointed_mobs = false;
        let transmitter_entity: Option<Entity> = None;

        spawn_mob(
            &mut cmds,
            &event.mob_ref,
            event.position,
            event.rotation,
            &mob_debug_settings,
            #[cfg(feature = "debug")]
            &logging_settings,
            &mob_registry,
            &game_assets,
            suppress_jointed_mobs,
            transmitter_entity,
            &mut spawner_effect_event_writer,
        )?;
    }
    Ok(())
}

/// Spawns a mob entity with all its components, decorations, and jointed sub-mobs
fn spawn_mob(
    cmds: &mut Commands,
    mob_ref: &MobRef,
    position: Vec2,
    rotation: f32,
    mob_debug_settings: &MobDebugSettings,
    #[cfg(feature = "debug")] logging_settings: &LoggingSettings,
    mob_registry: &MobRegistry,
    game_assets: &MergedGameAssets,
    suppress_jointed_mobs: bool,
    transmitter_entity: Option<Entity>, // entity that can transmit behaviors to the mob
    spawner_effect_event_writer: &mut MessageWriter<SpawnSpawnerEffectEvent>,
) -> Result<Entity, BevyError> {
    // MobRef is already normalized
    let normalized_ref = mob_ref.as_str();

    // Look up the mob from the registry (now returns &MobAsset directly)
    let mob = mob_registry
        .get_mob(normalized_ref)
        .ok_or(BevyError::from(format!(
            "Mob not found in registry: {}",
            normalized_ref
        )))?;

    // Load sprite using the path from the mob asset
    let sprite_handle = get_sprite_from_path(&mob.sprite, game_assets)?;

    // Spawn the main anchor entity with all core components
    let mut entity_commands = cmds.spawn((
        MobComponentBundle::from(mob),
        MobMarker::new(normalized_ref),
        AseAnimation {
            animation: Animation::tag("idle"),
            aseprite: sprite_handle,
        },
        Sprite::default(),
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        RigidBody::Dynamic,
        Transform::from_xyz(position.x, position.y, mob.z_level)
            .with_rotation(Quat::from_rotation_z(rotation.to_radians())),
    ));

    // Add collider bundle only if mob has colliders
    if let Some(collider_bundle) = MobColliderBundle::from_mob(mob) {
        entity_commands.insert(collider_bundle);
    }

    if let Some(mob_spawners) = &mob.mob_spawners {
        entity_commands.insert(mob_spawners.clone());
    }

    if let Some(entity) = transmitter_entity {
        entity_commands.insert(BehaviorReceiverComponent(entity));
    }

    let anchor_id = entity_commands
        .with_children(|parent| {
            // Spawn visual decorations as child entities
            for (decoration_sprite_path, pos) in &mob.decorations {
                parent.spawn((
                    Transform::from_xyz(pos.x, pos.y, 0.0),
                    AseAnimation {
                        animation: Animation::tag("idle"),
                        aseprite: match get_sprite_from_path(
                            decoration_sprite_path,
                            game_assets,
                        ) {
                            Ok(handle) => handle,
                            Err(_e) => {
                                thetawave_core::log_if!(
                                    logging_settings,
                                    spawning,
                                    warn,
                                    "Failed to load decoration sprite, skipping decoration: {}",
                                    _e
                                );
                                continue;
                            }
                        },
                    },
                    Sprite::default(),
                    Name::new(sprite_path_to_key(decoration_sprite_path).to_string()),
                ));
            }

            // Spawn behavior tree from registry
            if let Some(tree) = mob_registry.get_behavior(normalized_ref) {
                parent.spawn((
                    Name::new("Mob Behavior Tree"),
                    BehaveTree::new(tree.clone()),
                ));
            }
        })
        .id();

    // Set the transmitter entity for the spawned joints
    let new_transmitter_entity = if mob.behavior_transmitter {
        Some(anchor_id)
    } else {
        transmitter_entity
    };

    let mut mob_joints = HashMap::new();

    // Process all jointed sub-mobs (mobs connected via physics joints)
    for jointed_mob in &mob.jointed_mobs {
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
                    &jointed_mob.mob_ref, // Use mob_ref path
                    position + jointed_mob.offset_pos + chain.pos_offset * chain_index as f32,
                    0.0,
                    mob_debug_settings,
                    #[cfg(feature = "debug")]
                    logging_settings,
                    mob_registry,
                    game_assets,
                    chain_index < actual_length - 1, // Suppress jointed mobs except on the last chain link
                    new_transmitter_entity,
                    spawner_effect_event_writer,
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
                &jointed_mob.mob_ref, // Use mob_ref path
                position + jointed_mob.offset_pos,
                0.0,
                mob_debug_settings,
                #[cfg(feature = "debug")]
                logging_settings,
                mob_registry,
                game_assets,
                false,
                new_transmitter_entity,
                spawner_effect_event_writer,
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
    if let Some(ref mut projectile_spawners) = mob.projectile_spawners.clone() {
        for (key, spawner) in projectile_spawners.spawners.iter_mut() {
            spawner_effect_event_writer.write(SpawnSpawnerEffectEvent {
                parent_entity: anchor_id,
                effect_type: get_particle_effect_str(&spawner.projectile_type).to_string(),
                color: spawner.faction.get_color(),
                position: spawner.position,
                key: key.to_string(),
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
    jointed_mob: &JointedMobRef,
    anchor_offset: Vec2,
) -> Entity {
    // Create the revolute joint with anchor positions and compliance settings
    let mut joint = RevoluteJoint::new(anchor, jointed)
        .with_local_anchor1(jointed_mob.anchor_1_pos + anchor_offset)
        .with_local_anchor2(jointed_mob.anchor_2_pos)
        .with_point_compliance(jointed_mob.compliance);

    // Apply angle limits if specified (constrains how far the joint can rotate)
    if let Some(angle_limit_range) = &jointed_mob.angle_limit_range {
        joint.angle_limit = Some(AngleLimit::new(
            angle_limit_range.min.to_radians(),
            angle_limit_range.max.to_radians(),
        ));
        // Note: angle_limit_torque was removed in newer avian2d - torque is now set via compliance
    }
    // Spawn the joint entity into the world
    cmds.spawn(joint).id()
}

pub(crate) fn connect_effect_to_spawner(
    mut events: MessageReader<SpawnerParticleEffectSpawnedEvent>,
    mut mob_query: Query<&mut ProjectileSpawnerComponent, With<MobMarker>>,
    #[cfg(feature = "debug")] logging_settings: Res<LoggingSettings>,
) {
    for event in events.read() {
        // Directly access the parent mob using the parent_entity from the event
        if let Ok(mut projectile_spawner_component) = mob_query.get_mut(event.parent_entity) {
            // Check if this mob has a spawner with the matching key
            if let Some(spawner) = projectile_spawner_component.spawners.get_mut(&event.key) {
                spawner.spawn_effect_entity = Some(event.effect_entity);
            } else {
                thetawave_core::log_if!(
                    logging_settings,
                    spawning,
                    warn,
                    "Mob {} has no spawner with key '{}'",
                    event.parent_entity.index(),
                    event.key
                );
            }
        } else {
            thetawave_core::log_if!(
                logging_settings,
                spawning,
                warn,
                "Parent entity {} is not a valid mob with ProjectileSpawnerComponent",
                event.parent_entity.index()
            );
        }
    }
}
