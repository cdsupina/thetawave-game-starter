use bevy::{asset::Handle, image::Image, prelude::Resource};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_hui::prelude::HtmlTemplate;

// Main menu assets including HTML templates for various UI components
#[derive(AssetCollection, Resource)]
pub(crate) struct MainMenuAssets {
    // HTML template for main menu layout
    #[asset(path = "ui/menus/main_menu.html")]
    pub main_menu_html: Handle<HtmlTemplate>,
    // HTML template for menu buttons
    #[asset(path = "ui/components/menu_button.html")]
    pub menu_button_html: Handle<HtmlTemplate>,
    // HTML template for website footer button
    #[asset(path = "ui/components/website_footer_button.html")]
    pub website_footer_button_html: Handle<HtmlTemplate>,
}

// Assets for background images
#[derive(AssetCollection, Resource)]
pub(crate) struct BackgroundAssets {
    // Blue space background image
    #[asset(path = "media/images/backgrounds/blue.png")]
    pub blue_space_bg: Handle<Image>,
}
