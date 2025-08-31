use bevy::{
    asset::Handle,
    ecs::{
        event::EventReader,
        name::Name,
        system::{Commands, Res},
    },
    math::Quat,
    sprite::Sprite,
    transform::components::Transform,
};
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation, Aseprite};
use thetawave_assets::{asset_keys, AssetResolver, ExtendedGameAssets, GameAssets};
use thetawave_core::Faction;
use thetawave_states::{AppState, Cleanup};

use crate::{
    ProjectileType,
    attributes::{
        DespawnAfterAnimationComponent, ProjectileEffectType, SpawnProjectileEffectEvent,
    },
    spawn::FactionExt,
};

/// Get the Aseprite handle from ProjectileType and ProjectileEffectType using asset resolver
fn get_effect_sprite(
    projectile_type: &ProjectileType,
    effect_type: &ProjectileEffectType,
    extended_assets: &ExtendedGameAssets,
    game_assets: &GameAssets,
) -> Handle<Aseprite> {
    let key = match projectile_type {
        ProjectileType::Bullet => match effect_type {
            ProjectileEffectType::Despawn => asset_keys::BULLET_PROJECTILE_DESPAWN,
            ProjectileEffectType::Hit => asset_keys::BULLET_PROJECTILE_HIT,
        },
        ProjectileType::Blast => match effect_type {
            ProjectileEffectType::Despawn => asset_keys::BLAST_PROJECTILE_DESPAWN,
            ProjectileEffectType::Hit => asset_keys::BLAST_PROJECTILE_HIT,
        },
    };

    AssetResolver::get_sprite(key, extended_assets, game_assets)
        .unwrap_or_else(|| panic!("Missing sprite asset for projectile effect: {:?} {:?}", projectile_type, effect_type))
}

pub(crate) fn spawn_effect_system(
    mut cmds: Commands,
    game_assets: Res<GameAssets>,
    extended_assets: Res<ExtendedGameAssets>,
    mut spawn_projectile_effect_event_reader: EventReader<SpawnProjectileEffectEvent>,
) {
    for event in spawn_projectile_effect_event_reader.read() {
        spawn_effect(
            &mut cmds,
            &event.projectile_type,
            &event.effect_type,
            &event.faction,
            &event.transform,
            &game_assets,
            &extended_assets,
        );
    }
}

fn spawn_effect(
    cmds: &mut Commands,
    projectile_type: &ProjectileType,
    effect_type: &ProjectileEffectType,
    faction: &Faction,
    transform: &Transform,
    game_assets: &GameAssets,
    extended_assets: &ExtendedGameAssets,
) {
    cmds.spawn((
        Name::new("Projectile Effect"),
        faction.clone(),
        Sprite {
            color: faction.get_projectile_color(projectile_type),
            ..Default::default()
        },
        AseAnimation {
            animation: Animation::tag("idle"),
            aseprite: get_effect_sprite(projectile_type, effect_type, extended_assets, game_assets),
        },
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        (*transform).with_rotation(Quat::default()),
        DespawnAfterAnimationComponent,
    ));
}
