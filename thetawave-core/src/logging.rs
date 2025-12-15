#[cfg(feature = "debug")]
use bevy::ecs::resource::Resource;

/// Runtime logging settings, toggled via debug menu.
///
/// Each field controls whether logs for that category are emitted.
/// All categories are disabled by default.
///
/// Only available when the `debug` feature is enabled.
#[cfg(feature = "debug")]
#[derive(Resource, Default)]
pub struct LoggingSettings {
    /// Combat logs (projectile hits, damage values, health changes)
    pub combat: bool,
    /// Player ability execution and death events
    pub abilities: bool,
    /// Mob/entity spawning events and failures
    pub spawning: bool,
    /// Particle effect warnings and issues
    pub particles: bool,
    /// Data/asset loading events
    pub data: bool,
    /// UI interactions (website opens, options persistence)
    pub ui: bool,
}

/// Log an info message if the specified logging category is enabled.
/// Only compiles when the `debug` feature is enabled.
///
/// # Example
/// ```ignore
/// log_if!(logging_settings, combat, info, "Player hit for {} damage", damage);
/// ```
#[cfg(feature = "debug")]
#[macro_export]
macro_rules! log_if {
    ($settings:expr, $category:ident, info, $($arg:tt)*) => {
        if $settings.$category {
            bevy::log::info!($($arg)*);
        }
    };
    ($settings:expr, $category:ident, warn, $($arg:tt)*) => {
        if $settings.$category {
            bevy::log::warn!($($arg)*);
        }
    };
    ($settings:expr, $category:ident, error, $($arg:tt)*) => {
        if $settings.$category {
            bevy::log::error!($($arg)*);
        }
    };
    ($settings:expr, $category:ident, debug, $($arg:tt)*) => {
        if $settings.$category {
            bevy::log::debug!($($arg)*);
        }
    };
    ($settings:expr, $category:ident, trace, $($arg:tt)*) => {
        if $settings.$category {
            bevy::log::trace!($($arg)*);
        }
    };
}

/// No-op version of log_if! when debug feature is disabled.
/// This ensures the macro call compiles but produces no code.
#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! log_if {
    ($settings:expr, $category:ident, $level:ident, $($arg:tt)*) => {};
}
