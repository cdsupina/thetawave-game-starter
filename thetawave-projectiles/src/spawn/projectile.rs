use avian2d::prelude::{
    Collider, CollisionEventsEnabled, CollisionLayers, LinearVelocity, MassPropertiesBundle,
    PhysicsLayer, RigidBody, Sensor,
};
use bevy::{
    asset::Handle,
    ecs::{
        entity::Entity,
        error::{BevyError, Result},
        message::{MessageReader, MessageWriter},
        name::Name,
        system::{Commands, Res},
    },
    math::{Quat, Vec2},
    sprite::Sprite,
    transform::components::Transform,
};
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation, Aseprite};
use thetawave_assets::{AssetError, AssetResolver, ExtendedGameAssets, GameAssets};
use thetawave_core::{AppState, Cleanup};
use thetawave_core::{CollisionDamage, Faction};
use thetawave_particles::SpawnProjectileTrailEffectEvent;
use thetawave_physics::ThetawavePhysicsLayer;

use crate::{
    ProjectileType, SpawnProjectileEvent,
    attributes::{ProjectileAttributesResource, ProjectileRangeComponent, ProjectileSpread},
    spawn::FactionExt,
};
use rand::{Rng, rng};

/// Get the collision layer membership bits for projectiles of the given faction
fn get_projectile_collision_membership(faction: &Faction) -> u32 {
    match faction {
        Faction::Ally => ThetawavePhysicsLayer::AllyProjectile.to_bits(),
        Faction::Enemy => ThetawavePhysicsLayer::EnemyProjectile.to_bits(),
    }
}

/// Get the collision layer filter bits for what projectiles of the given faction can collide with
fn get_projectile_collision_filter(faction: &Faction) -> u32 {
    match faction {
        Faction::Ally => {
            ThetawavePhysicsLayer::EnemyMob.to_bits()
                | ThetawavePhysicsLayer::EnemyTentacle.to_bits()
        }
        Faction::Enemy => {
            ThetawavePhysicsLayer::AllyMob.to_bits() | ThetawavePhysicsLayer::Player.to_bits()
        }
    }
}

/// Get the Aseprite handle from a given ProjectileType using asset resolver
fn get_projectile_sprite(
    projectile_type: &ProjectileType,
    extended_assets: &ExtendedGameAssets,
    game_assets: &GameAssets,
) -> Result<Handle<Aseprite>, AssetError> {
    let key = match projectile_type {
        ProjectileType::Bullet => "bullet_projectile",
        ProjectileType::Blast => "blast_projectile",
    };

    AssetResolver::get_game_sprite(key, extended_assets, game_assets)
}

/// Calculate spread velocities based on the spread pattern
fn calculate_spread_velocities(
    base_velocity: Vec2,
    count: u8,
    spread_pattern: &ProjectileSpread,
) -> Vec<Vec2> {
    if count == 0 {
        return vec![];
    }

    if count == 1 {
        return vec![base_velocity];
    }

    match spread_pattern {
        ProjectileSpread::Arc {
            max_spread,
            projectile_gap,
            spread_weights,
        } => {
            let speed = base_velocity.length();
            let base_angle = base_velocity.y.atan2(base_velocity.x);

            // Convert degrees to radians
            let max_spread_rad = max_spread.to_radians();
            let projectile_gap_rad = projectile_gap.to_radians();

            // Calculate the angle segment between projectiles
            let spread_angle_segment = max_spread_rad
                .min(projectile_gap_rad * (count as f32 - 1.0))
                / (count as f32 - 1.0).max(1.0);

            let mut velocities = Vec::new();

            for p in 0..count {
                // Calculate angle offset from center
                let angle_offset = (p as f32 - (count as f32 - 1.0) / 2.0) * spread_angle_segment;
                let projectile_angle = base_angle + angle_offset;

                // Calculate speed variation based on distance from center
                let center_distance = if count <= 1 {
                    0.0
                } else {
                    // Normalized distance from center (0.0 at center, 1.0 at edges)
                    (p as f32 - (count as f32 - 1.0) / 2.0).abs() / ((count as f32 - 1.0) / 2.0)
                };

                // Apply spread weights: 1.0 = uniform, >1.0 = faster center, <1.0 = slower center
                let speed_multiplier = if *spread_weights == 1.0 {
                    1.0
                } else if *spread_weights > 1.0 {
                    // Faster center, slower edges: lerp from spread_weights (center) to 1.0 (edges)
                    spread_weights * (1.0 - center_distance) + 1.0 * center_distance
                } else {
                    // Slower center, faster edges: lerp from spread_weights (center) to (2.0 - spread_weights) (edges)
                    // This ensures that when spread_weights = 0.5, center gets 0.5x speed and edges get 1.5x speed
                    let edge_multiplier = 2.0 - spread_weights;
                    spread_weights * (1.0 - center_distance) + edge_multiplier * center_distance
                };

                let velocity = Vec2::from_angle(projectile_angle) * speed * speed_multiplier;
                velocities.push(velocity);
            }

            velocities
        }
        ProjectileSpread::Random {
            max_spread,
            speed_variance,
        } => {
            let mut rng = rng();
            let mut velocities = Vec::new();

            for _ in 0..count {
                // Random angle within ±(max_spread/2) degrees
                let half_spread = max_spread / 2.0;
                let random_angle_deg = rng.random_range(-half_spread..=half_spread);
                // Random speed multiplier: 1.0 ± speed_variance (e.g., 1.0 ± 0.2 = 0.8 to 1.2)
                let random_speed_multiplier =
                    rng.random_range((1.0 - speed_variance)..=(1.0 + speed_variance));

                let base_angle = base_velocity.y.atan2(base_velocity.x);
                // Convert degrees to radians
                let projectile_angle = base_angle + random_angle_deg.to_radians();
                let projectile_speed = base_velocity.length() * random_speed_multiplier;

                let velocity = Vec2::from_angle(projectile_angle) * projectile_speed;
                velocities.push(velocity);
            }

            velocities
        }
    }
}

pub(crate) fn spawn_projectile_system(
    mut cmds: Commands,
    game_assets: Res<GameAssets>,
    extended_assets: Res<ExtendedGameAssets>,
    mut spawn_projectile_event_reader: MessageReader<SpawnProjectileEvent>,
    attributes_res: Res<ProjectileAttributesResource>,
    mut projectile_trail_effect_event_writer: MessageWriter<SpawnProjectileTrailEffectEvent>,
) -> Result {
    for event in spawn_projectile_event_reader.read() {
        let _spawned_entities = spawn_projectile(
            &mut cmds,
            &event.projectile_type,
            &event.projectile_spread,
            event.count,
            &event.faction,
            event.position,
            event.scale,
            event.velocity,
            event.damage,
            event.range_seconds,
            &game_assets,
            &extended_assets,
            &attributes_res,
            &mut projectile_trail_effect_event_writer,
        )?;
    }

    Ok(())
}

fn spawn_projectile(
    cmds: &mut Commands,
    projectile_type: &ProjectileType,
    projectile_spread: &ProjectileSpread,
    count: u8,
    faction: &Faction,
    position: Vec2,
    scale: f32,
    velocity: Vec2,
    damage: u32,
    range_seconds: f32,
    game_assets: &GameAssets,
    extended_assets: &ExtendedGameAssets,
    attributes_res: &ProjectileAttributesResource,
    projectile_trail_effect_event_writer: &mut MessageWriter<SpawnProjectileTrailEffectEvent>,
) -> Result<Vec<Entity>, BevyError> {
    let collision_layers = CollisionLayers::new(
        get_projectile_collision_membership(faction),
        get_projectile_collision_filter(faction),
    );

    // Look up the projectiles's configuration data from resources
    let projectile_attributes = attributes_res
        .attributes
        .get(projectile_type)
        .ok_or(BevyError::from("Projectile attributes not found"))?;

    // Calculate spread velocities for all projectiles
    let velocities = calculate_spread_velocities(velocity, count, projectile_spread);
    let mut spawned_entities = Vec::new();

    for projectile_velocity in velocities {
        // Calculate the projectile's rotation from its velocity vector
        let rotation = projectile_velocity.y.atan2(projectile_velocity.x);

        // Spawn the projectile
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
                aseprite: get_projectile_sprite(projectile_type, extended_assets, game_assets)?,
            },
            RigidBody::Dynamic,
            collision_layers,
            Cleanup::<AppState> {
                states: vec![AppState::Game],
            },
            Transform::from_xyz(position.x, position.y, 0.0)
                .with_rotation(Quat::from_rotation_z(rotation))
                .with_scale(Vec2::splat(scale).extend(1.0)),
            LinearVelocity(projectile_velocity),
            CollisionEventsEnabled,
            CollisionDamage(damage),
            ProjectileRangeComponent::new(range_seconds),
        ));

        // Add the sensor component for projectiles that are not "physical"
        // Sensors don't contribute mass in avian2d 0.4+, so we add MassPropertiesBundle explicitly
        if projectile_attributes.is_sensor {
            entity_cmds.insert((
                Sensor,
                MassPropertiesBundle::from_shape(&Collider::from(projectile_attributes), 1.0),
            ));
        }

        let particle_entity = entity_cmds.id();

        spawned_entities.push(particle_entity);

        // Spawn the particle trail effect
        projectile_trail_effect_event_writer.write(SpawnProjectileTrailEffectEvent {
            color: faction.get_color(),
            parent_entity: particle_entity,
            scale,
        });
    }

    Ok(spawned_entities)
}
