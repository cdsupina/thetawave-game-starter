mod asset;
mod attributes;
mod behavior;
mod spawn;
mod systems;

// Only export types that are used by other crates
pub use asset::MobRegistry;
pub use attributes::{JointsComponent, MobAttributesComponent, MobDeathEvent, MobMarker};
pub use behavior::{BehaviorActionName, BehaviorReceiverComponent, MobBehaviorComponent};
pub use spawn::MobDebugSettings;

// Internal imports for this crate
use asset::{
    ExtendedMobPatches, ExtendedMobs, MobAssetLoader, MobAssets, MobPatch, MobPatchLoader, RawMob,
};

use bevy::{
    app::{Plugin, Update},
    asset::{AssetApp, Assets},
    ecs::{
        schedule::{IntoScheduleConfigs, SystemCondition, common_conditions::on_message},
        system::{Commands, Res},
    },
    prelude::OnEnter,
    state::condition::in_state,
};
use bevy_asset_loader::{
    loading_state::{config::ConfigureLoadingState, LoadingStateAppExt},
    prelude::LoadingStateConfig,
    standard_dynamic_asset::StandardDynamicAssetCollection,
};
use thetawave_core::{AppState, GameState};
use thetawave_particles::SpawnerParticleEffectSpawnedEvent;

use crate::{
    attributes::ThetawaveAttributesPlugin,
    behavior::ThetawaveMobBehaviorPlugin,
    spawn::{connect_effect_to_spawner, spawn_mob_system},
    systems::{joint_bleed_system, mob_death_system},
};

pub use spawn::SpawnMobEvent;

pub struct ThetawaveMobsPlugin;

impl Plugin for ThetawaveMobsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Register asset types and loaders
        app.init_asset::<RawMob>()
            .init_asset_loader::<MobAssetLoader>()
            .init_asset::<MobPatch>()
            .init_asset_loader::<MobPatchLoader>();

        // Configure mob asset loading during game loading state
        app.configure_loading_state(
            LoadingStateConfig::new(AppState::GameLoading)
                // Load base .mob files
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("mobs.assets.ron")
                .load_collection::<MobAssets>()
                // Load extended .mob and .mobpatch files
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "extended://mobs.assets.ron",
                )
                .load_collection::<ExtendedMobs>()
                .load_collection::<ExtendedMobPatches>(),
        );

        #[cfg(feature = "debug")]
        app.insert_resource(MobDebugSettings::default());

        app.add_plugins((ThetawaveMobBehaviorPlugin, ThetawaveAttributesPlugin));

        // Build the MobRegistry when entering Game state (after assets are loaded)
        app.add_systems(OnEnter(AppState::Game), build_mob_registry_system);

        app.add_systems(
            Update,
            (
                spawn_mob_system.run_if(on_message::<SpawnMobEvent>),
                connect_effect_to_spawner.run_if(on_message::<SpawnerParticleEffectSpawnedEvent>),
                (joint_bleed_system, mob_death_system)
                    .chain()
                    .run_if(on_message::<MobDeathEvent>),
            )
                .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
        )
        .add_message::<SpawnMobEvent>();
    }
}

/// System to build the MobRegistry from loaded RawMob assets and MobPatches
fn build_mob_registry_system(
    mut commands: Commands,
    mob_assets_collection: Res<MobAssets>,
    extended_mobs: Res<ExtendedMobs>,
    extended_mob_patches: Res<ExtendedMobPatches>,
    raw_mob_assets: Res<Assets<RawMob>>,
    mob_patches: Res<Assets<MobPatch>>,
) {
    let registry = MobRegistry::build(
        &mob_assets_collection,
        &extended_mobs,
        &extended_mob_patches,
        &raw_mob_assets,
        &mob_patches,
    );
    commands.insert_resource(registry);
}
