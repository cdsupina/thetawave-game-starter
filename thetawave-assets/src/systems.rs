use crate::ExtendedGameAssets;

use super::data::ParticleMaterials;
use super::{GameAssets, LoadingProgressEvent};
use bevy::{
    asset::Assets,
    ecs::system::Commands,
    prelude::{EventWriter, Res, ResMut},
};
use bevy_enoki::prelude::ColorParticle2dMaterial;
use iyes_progress::ProgressTracker;
use thetawave_core::Faction;
use thetawave_states::AppState;

/// System for getting loading progress and sending the value as an event
pub(super) fn get_loading_progress_system(
    progress: Res<ProgressTracker<AppState>>,
    mut loading_event_writer: EventWriter<LoadingProgressEvent>,
) {
    let progress = progress.get_global_progress();
    loading_event_writer.write(LoadingProgressEvent(
        progress.done as f32 / progress.total as f32,
    ));
}

/// Setup particle materials with faction-specific colors
/// Should be called when entering the game loading state
pub(super) fn setup_particle_materials_system(
    mut cmds: Commands,
    mut materials: ResMut<Assets<ColorParticle2dMaterial>>,
) {
    // Create faction-specific color materials
    let ally_material = materials.add(ColorParticle2dMaterial::new(
        Faction::Ally.get_base_color().into(),
    ));

    let enemy_material = materials.add(ColorParticle2dMaterial::new(
        Faction::Enemy.get_base_color().into(),
    ));

    // Insert the ParticleMaterials resource
    cmds.insert_resource(ParticleMaterials {
        ally_material,
        enemy_material,
    });
}

/// Unload game assets resource
/// Should be called once when exiting the game app state
pub(super) fn unload_game_assets_system(mut cmds: Commands) {
    cmds.remove_resource::<GameAssets>();
    cmds.remove_resource::<ExtendedGameAssets>();
    cmds.remove_resource::<ParticleMaterials>();
}
