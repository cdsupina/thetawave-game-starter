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
use thetawave_assets::GameAssets;
use thetawave_core::Faction;
use thetawave_states::{AppState, Cleanup};

use crate::{
    ProjectileType,
    attributes::{DespawnAfterAnimationComponent, SpawnProjectileEffectEvent},
    spawn::FactionExt,
};

trait GameAssetsExt {
    fn get_effect_sprite(&self, projectile_type: &ProjectileType) -> Handle<Aseprite>;
}

impl GameAssetsExt for GameAssets {
    fn get_effect_sprite(&self, projectile_type: &ProjectileType) -> Handle<Aseprite> {
        match projectile_type {
            ProjectileType::Bullet => self.bullet_projectile_despawn_aseprite.clone(),
            ProjectileType::Blast => self.blast_projectile_despawn_aseprite.clone(),
        }
    }
}

pub(crate) fn spawn_effect_system(
    mut cmds: Commands,
    assets: Res<GameAssets>,
    mut spawn_projectile_effect_event_reader: EventReader<SpawnProjectileEffectEvent>,
) {
    for event in spawn_projectile_effect_event_reader.read() {
        spawn_effect(
            &mut cmds,
            &event.projectile_type,
            &event.faction,
            &event.transform,
            &assets,
        );
    }
}

fn spawn_effect(
    cmds: &mut Commands,
    projectile_type: &ProjectileType,
    faction: &Faction,
    transform: &Transform,
    assets: &GameAssets,
) {
    cmds.spawn((
        Name::new("Projectile Effect"),
        //projectile_type.clone(),
        faction.clone(),
        Sprite {
            color: faction.get_projectile_color(projectile_type),
            ..Default::default()
        },
        AseAnimation {
            animation: Animation::tag("idle"),
            aseprite: assets.get_effect_sprite(projectile_type),
        },
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        (*transform).with_rotation(Quat::default()),
        DespawnAfterAnimationComponent,
    ));
}
