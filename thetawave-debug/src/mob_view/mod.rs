//! Mob View Debug Window
//!
//! A secondary debug window for inspecting mobs in real-time during gameplay.
//!
//! ## Features
//!
//! - Camera follows selected mob group with zoom controls (+/-)
//! - Left panel shows stats: health, movement, physics, combat for all connected mobs
//! - Right panel visualizes behavior trees with active node highlighting
//! - Tab key cycles through all mob groups
//!
//! ## Mob Grouping Algorithm
//!
//! Connected mobs (via physics joints) are treated as a single unit. The algorithm:
//! 1. Builds an adjacency graph from all `RevoluteJoint` entities
//! 2. Uses BFS to find connected components of mobs
//! 3. Selects a "group key" entity (preferring mobs with `JointsComponent`)
//! 4. When joints break, groups automatically split on the next frame
//!
//! ## Particle Rendering Workaround
//!
//! Due to bevy_enoki limitations with multiple cameras, particle rendering is
//! disabled while the mob view window is open. The `ParticleRenderingEnabled`
//! resource controls this behavior and is automatically toggled when the
//! window opens/closes.

mod behavior;
mod camera;
mod data;
mod groups;
mod plugin;
mod selection;
mod stats;
mod ui;
mod window;

pub use plugin::MobViewPlugin;
pub use window::ToggleMobViewWindowEvent;
