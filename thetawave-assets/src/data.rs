use bevy::{
    asset::Handle,
    image::Image,
    platform::collections::HashMap,
    prelude::{Event, Res, Resource},
    scene::Scene,
    text::Font,
};
use bevy_aseprite_ultra::prelude::Aseprite;
use bevy_asset_loader::{asset_collection::AssetCollection, mapped::AssetFileStem};
use bevy_enoki::{Particle2dEffect, prelude::ColorParticle2dMaterial};
use bevy_kira_audio::AudioSource;
use rand::Rng;
use thetawave_core::Faction;

/// Assets used in the game state
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(key = "game_sprites", collection(typed, mapped))]
    pub sprites: HashMap<AssetFileStem, Handle<Aseprite>>,
    #[asset(key = "game_particle_effects", collection(typed, mapped))]
    pub particle_effects: HashMap<AssetFileStem, Handle<Particle2dEffect>>,
}

/// Additional assets used in the game state that are not built in to thetawave-assets
#[derive(AssetCollection, Resource, Default, Clone)]
pub struct ExtendedGameAssets {
    #[asset(key = "extended_game_sprites", collection(typed, mapped))]
    pub sprites: HashMap<AssetFileStem, Handle<Aseprite>>,
    #[asset(key = "extended_game_particle_effects", collection(typed, mapped))]
    pub particle_effects: HashMap<AssetFileStem, Handle<Particle2dEffect>>,
}

/// Resource for storing faction-based particle materials
#[derive(Resource)]
pub struct ParticleMaterials {
    pub ally_material: Handle<ColorParticle2dMaterial>,
    pub enemy_material: Handle<ColorParticle2dMaterial>,
}

impl ParticleMaterials {
    pub fn get_material_for_faction(&self, faction: &Faction) -> Handle<ColorParticle2dMaterial> {
        match faction {
            Faction::Ally => self.ally_material.clone(),
            Faction::Enemy => self.enemy_material.clone(),
        }
    }
}

/// Utility for resolving assets with ExtendedGameAssets priority and GameAssets fallback
///
/// Note: With bevy_asset_loader's `Files(paths: [...])` format and `collection(typed, mapped)`,
/// assets are keyed by their full file path as specified in the paths array.
/// For example, "media/aseprite/bullet_projectile.aseprite" gets key "media/aseprite/bullet_projectile.aseprite".
pub struct AssetResolver;

impl AssetResolver {
    /// Get an Aseprite handle by key, checking ExtendedGameAssets first, then GameAssets
    pub fn get_game_sprite(
        key: &str,
        extended_assets: &ExtendedGameAssets,
        game_assets: &GameAssets,
    ) -> Handle<Aseprite> {
        extended_assets
            .sprites
            .get(key)
            .or_else(|| game_assets.sprites.get(key))
            .cloned()
            .unwrap_or_else(|| panic!("Missing sprite asset for key: {:?}", key))
    }

    /// Get an Particle2DEffect handle by key, checking ExtendedGameAssets first, then GameAssets
    pub fn get_game_particle_effect(
        key: &str,
        extended_assets: &ExtendedGameAssets,
        game_assets: &GameAssets,
    ) -> Handle<Particle2dEffect> {
        extended_assets
            .particle_effects
            .get(key)
            .or_else(|| game_assets.particle_effects.get(key))
            .cloned()
            .unwrap_or_else(|| panic!("Missing particle asset for key: {:?}", key))
    }

    pub fn get_ui_image(key: &str, ui_assets: &UiAssets) -> Handle<Image> {
        ui_assets
            .images
            .get(key)
            .cloned()
            .unwrap_or_else(|| panic!("Missing image asset for key: {:?}", key))
    }

    pub fn get_ui_sprite(key: &str, ui_assets: &UiAssets) -> Handle<Aseprite> {
        ui_assets
            .sprites
            .get(key)
            .cloned()
            .unwrap_or_else(|| panic!("Missing sprite asset for key: {:?}", key))
    }

    pub fn get_ui_font(key: &str, ui_assets: &UiAssets) -> Handle<Font> {
        ui_assets
            .fonts
            .get(key)
            .cloned()
            .unwrap_or_else(|| panic!("Missing font asset for key: {:?}", key))
    }
}

/// Audio assets used throughout all states of the app
#[derive(AssetCollection, Resource)]
pub struct MusicAssets {
    #[asset(key = "main_menu_theme")]
    pub main_menu_theme: Handle<AudioSource>,
    #[asset(key = "game_theme")]
    pub game_theme: Handle<AudioSource>,
}

// Assets for Bevy ui
#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(key = "ui_sprites", collection(typed, mapped))]
    pub sprites: HashMap<AssetFileStem, Handle<Aseprite>>,
    #[asset(key = "ui_images", collection(typed, mapped))]
    pub images: HashMap<AssetFileStem, Handle<Image>>,
    #[asset(key = "ui_fonts", collection(typed, mapped))]
    pub fonts: HashMap<AssetFileStem, Handle<Font>>,
    #[asset(key = "ui_button_select_audio", collection(typed))]
    pub menu_button_select_effects: Vec<Handle<AudioSource>>,
    #[asset(key = "ui_button_release_audio", collection(typed))]
    pub menu_button_release_effects: Vec<Handle<AudioSource>>,
    #[asset(key = "ui_button_confirm_audio", collection(typed))]
    pub menu_button_confirm_effects: Vec<Handle<AudioSource>>,
}

impl UiAssets {
    pub fn get_random_button_press_effect(&self) -> Handle<AudioSource> {
        self.menu_button_select_effects
            [rand::rng().random_range(0..self.menu_button_select_effects.len())]
        .clone()
    }

    pub fn get_random_button_release_effect(&self) -> Handle<AudioSource> {
        self.menu_button_release_effects
            [rand::rng().random_range(0..self.menu_button_release_effects.len())]
        .clone()
    }

    pub fn get_random_button_confirm_effect(&self) -> Handle<AudioSource> {
        self.menu_button_confirm_effects
            [rand::rng().random_range(0..self.menu_button_confirm_effects.len())]
        .clone()
    }
}

// Assets for background images
#[derive(AssetCollection, Resource)]
pub struct BackgroundAssets {
    // all space backgrounds
    #[asset(
        paths(
            "media/images/backgrounds/blue.png",
            "media/images/backgrounds/blue_green.png",
            "media/images/backgrounds/blue_purple.png",
            "media/images/backgrounds/blue_yellow.png",
            "media/images/backgrounds/deasaturated_red.png",
            "media/images/backgrounds/gray.png",
            "media/images/backgrounds/green.png",
            "media/images/backgrounds/green_blue.png",
            "media/images/backgrounds/green_purple.png",
            "media/images/backgrounds/green_yellow.png",
            "media/images/backgrounds/purple.png",
            "media/images/backgrounds/purple_blue.png",
            "media/images/backgrounds/purple_green.png",
            "media/images/backgrounds/purple_yellow.png",
            "media/images/backgrounds/red.png",
            "media/images/backgrounds/red_purple.png",
            "media/images/backgrounds/red_purple2.png",
            "media/images/backgrounds/red_yellow.png",
            "media/images/backgrounds/yellow.png",
        ),
        collection(typed)
    )]
    pub space_backgrounds: Vec<Handle<Image>>,
    // all planets
    #[asset(
        paths(
            "media/models/planets/Planet_1.glb#Scene0",
            "media/models/planets/Planet_2.glb#Scene0",
            "media/models/planets/Planet_3.glb#Scene0",
            "media/models/planets/Planet_4.glb#Scene0",
            "media/models/planets/Planet_5.glb#Scene0",
            "media/models/planets/Planet_6.glb#Scene0",
            "media/models/planets/Planet_7.glb#Scene0",
            "media/models/planets/Planet_8.glb#Scene0",
            "media/models/planets/Planet_9.glb#Scene0",
            "media/models/planets/Planet_10.glb#Scene0",
            "media/models/planets/Planet_11.glb#Scene0",
            "media/models/planets/Planet_12.glb#Scene0",
            "media/models/planets/Planet_13.glb#Scene0",
            "media/models/planets/Planet_14.glb#Scene0",
            "media/models/planets/Planet_15.glb#Scene0",
            "media/models/planets/Planet_16.glb#Scene0",
            "media/models/planets/Planet_17.glb#Scene0",
            "media/models/planets/Planet_18.glb#Scene0",
            "media/models/planets/Planet_19.glb#Scene0",
            "media/models/planets/Planet_20.glb#Scene0",
            "media/models/planets/Planet_21.glb#Scene0",
            "media/models/planets/Planet_22.glb#Scene0",
            "media/models/planets/Planet_23.glb#Scene0",
            "media/models/planets/Planet_24.glb#Scene0",
            "media/models/planets/Planet_25.glb#Scene0",
            "media/models/planets/Planet_26.glb#Scene0",
            "media/models/planets/Planet_27.glb#Scene0",
            "media/models/planets/Planet_28.glb#Scene0",
            "media/models/planets/Planet_29.glb#Scene0",
            "media/models/planets/Planet_30.glb#Scene0",
            "media/models/planets/Planet_31.glb#Scene0",
            "media/models/planets/Planet_32.glb#Scene0",
            "media/models/planets/Planet_33.glb#Scene0",
            "media/models/planets/Planet_34.glb#Scene0",
            "media/models/planets/Planet_35.glb#Scene0",
            "media/models/planets/Planet_36.glb#Scene0",
            "media/models/planets/Planet_37.glb#Scene0",
            "media/models/planets/Planet_38.glb#Scene0",
            "media/models/planets/Planet_39.glb#Scene0",
            "media/models/planets/Planet_40.glb#Scene0",
            "media/models/planets/Planet_41.glb#Scene0",
            "media/models/planets/Planet_42.glb#Scene0",
            "media/models/planets/Planet_43.glb#Scene0",
            "media/models/planets/Planet_44.glb#Scene0",
            "media/models/planets/Planet_45.glb#Scene0",
            "media/models/planets/Planet_46.glb#Scene0",
            "media/models/planets/Planet_47.glb#Scene0",
            "media/models/planets/Planet_48.glb#Scene0",
            "media/models/planets/Planet_49.glb#Scene0",
            "media/models/planets/Planet_50.glb#Scene0",
            "media/models/planets/Planet_51.glb#Scene0",
            "media/models/planets/Planet_52.glb#Scene0",
            "media/models/planets/Planet_53.glb#Scene0",
            "media/models/planets/Planet_54.glb#Scene0",
            "media/models/planets/Planet_55.glb#Scene0",
            "media/models/planets/Planet_56.glb#Scene0",
            "media/models/planets/Planet_57.glb#Scene0",
            "media/models/planets/Planet_58.glb#Scene0",
            "media/models/planets/Planet_59.glb#Scene0",
            "media/models/planets/Planet_60.glb#Scene0",
            "media/models/planets/Planet_61.glb#Scene0",
            "media/models/planets/Planet_62.glb#Scene0",
            "media/models/planets/Planet_63.glb#Scene0",
            "media/models/planets/Planet_64.glb#Scene0",
            "media/models/planets/Planet_65.glb#Scene0",
            "media/models/planets/Planet_66.glb#Scene0",
            "media/models/planets/Planet_67.glb#Scene0",
            "media/models/planets/Planet_68.glb#Scene0",
            "media/models/planets/Planet_69.glb#Scene0",
            "media/models/planets/Planet_70.glb#Scene0",
            "media/models/planets/Planet_71.glb#Scene0",
            "media/models/planets/Planet_72.glb#Scene0",
            "media/models/planets/Planet_73.glb#Scene0",
            "media/models/planets/Planet_74.glb#Scene0",
            "media/models/planets/Planet_75.glb#Scene0",
            "media/models/planets/Planet_76.glb#Scene0",
            "media/models/planets/Planet_77.glb#Scene0",
            "media/models/planets/Planet_78.glb#Scene0",
            "media/models/planets/Planet_79.glb#Scene0",
            "media/models/planets/Planet_80.glb#Scene0",
            "media/models/planets/Planet_81.glb#Scene0",
            "media/models/planets/Planet_82.glb#Scene0",
            "media/models/planets/Planet_83.glb#Scene0",
            "media/models/planets/Planet_84.glb#Scene0",
            "media/models/planets/Planet_85.glb#Scene0",
            "media/models/planets/Planet_86.glb#Scene0",
            "media/models/planets/Planet_87.glb#Scene0",
            "media/models/planets/Planet_88.glb#Scene0",
            "media/models/planets/Planet_89.glb#Scene0",
            "media/models/planets/Planet_90.glb#Scene0",
            "media/models/planets/Planet_91.glb#Scene0",
            "media/models/planets/Planet_92.glb#Scene0",
            "media/models/planets/Planet_93.glb#Scene0",
            "media/models/planets/Planet_94.glb#Scene0",
            "media/models/planets/Planet_95.glb#Scene0",
            "media/models/planets/Planet_96.glb#Scene0",
            "media/models/planets/Planet_97.glb#Scene0",
            "media/models/planets/Planet_98.glb#Scene0",
            "media/models/planets/Planet_99.glb#Scene0",
            "media/models/planets/Planet_100.glb#Scene0",
            "media/models/planets/Planet_101.glb#Scene0",
        ),
        collection(typed)
    )]
    pub planets: Vec<Handle<Scene>>,
}

impl BackgroundAssets {
    pub fn get_random_space_bg(&self) -> Handle<Image> {
        self.space_backgrounds[rand::rng().random_range(0..self.space_backgrounds.len())].clone()
    }

    pub fn get_random_planet(&self) -> Handle<Scene> {
        self.planets[rand::rng().random_range(0..self.planets.len())].clone()
    }
}

/// Event for sending percentage of loading progress
#[derive(Event)]
pub struct LoadingProgressEvent(pub f32);
