use bevy::{
    ecs::{entity::Entity, message::Message},
    prelude::Component,
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use strum::EnumProperty;
use strum_macros::{AsRefStr, EnumDiscriminants, EnumIter, EnumProperty, EnumString};

/// Used for receiving behaviors from another mob's TransmitMobBehavior
/// The entity is the mob entity that behaviors can be receieved from
#[derive(Component, Reflect)]
pub struct BehaviorReceiverComponent(pub Entity);

/// Target component used for behaviors that target other entities
/// Such as homing missiles
#[derive(Component, Reflect)]
pub(crate) struct TargetComponent(pub Entity);

/// Category of mob behavior types for UI organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MobBehaviorCategory {
    Movement,
    Targeting,
    Spawning,
    Timing,
    Communication,
    Joints,
}

impl MobBehaviorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Movement => "Movement",
            Self::Targeting => "Targeting",
            Self::Spawning => "Spawning",
            Self::Timing => "Timing",
            Self::Communication => "Communication",
            Self::Joints => "Joints",
        }
    }
}

/// Mob behaviors that can be run together at a single node in the behavior tree
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(Hash, EnumIter, AsRefStr, EnumString, EnumProperty))]
#[strum_discriminants(name(MobBehaviorVariant))]
#[serde(tag = "action", deny_unknown_fields)]
pub enum MobBehaviorType {
    // Movement behaviors
    #[strum(props(category = "Movement"))]
    MoveDown,
    #[strum(props(category = "Movement"))]
    #[allow(dead_code)] // Available for future use in behavior trees
    MoveUp,
    #[strum(props(category = "Movement"))]
    MoveLeft,
    #[strum(props(category = "Movement"))]
    MoveRight,
    #[strum(props(category = "Movement"))]
    BrakeHorizontal,
    #[strum(props(category = "Movement"))]
    BrakeAngular,
    #[strum(props(category = "Movement"))]
    MoveTo { x: f32, y: f32 },
    #[strum(props(category = "Movement"))]
    MoveForward,

    // Targeting behaviors
    #[strum(props(category = "Targeting"))]
    FindPlayerTarget,
    #[strum(props(category = "Targeting"))]
    MoveToTarget,
    #[strum(props(category = "Targeting"))]
    RotateToTarget,
    #[strum(props(category = "Targeting"))]
    LoseTarget,

    // Spawning behaviors
    #[strum(props(category = "Spawning"))]
    SpawnMob { keys: Option<Vec<String>> },
    #[strum(props(category = "Spawning"))]
    SpawnProjectile { keys: Option<Vec<String>> },

    // Timing behaviors
    #[strum(props(category = "Timing"))]
    DoForTime { seconds: f32 },

    // Communication behaviors
    #[strum(props(category = "Communication"))]
    TransmitMobBehavior {
        mob_type: String,
        behaviors: Vec<MobBehaviorType>,
    },

    // Joint behaviors
    #[strum(props(category = "Joints"))]
    #[allow(dead_code)]
    RotateJointsClockwise { keys: Vec<String> },
}

/// All available behavior variants organized by category.
/// Generated dynamically from the `#[strum(props(category = "..."))]` annotations.
pub static BY_CATEGORY: LazyLock<Vec<(MobBehaviorCategory, Vec<MobBehaviorVariant>)>> =
    LazyLock::new(|| {
        use std::collections::HashMap;
        use strum::IntoEnumIterator;

        let mut map: HashMap<MobBehaviorCategory, Vec<MobBehaviorVariant>> = HashMap::new();
        for variant in MobBehaviorVariant::iter() {
            map.entry(variant.category()).or_default().push(variant);
        }

        // Return in consistent display order
        vec![
            (
                MobBehaviorCategory::Movement,
                map.remove(&MobBehaviorCategory::Movement)
                    .unwrap_or_default(),
            ),
            (
                MobBehaviorCategory::Targeting,
                map.remove(&MobBehaviorCategory::Targeting)
                    .unwrap_or_default(),
            ),
            (
                MobBehaviorCategory::Spawning,
                map.remove(&MobBehaviorCategory::Spawning)
                    .unwrap_or_default(),
            ),
            (
                MobBehaviorCategory::Timing,
                map.remove(&MobBehaviorCategory::Timing).unwrap_or_default(),
            ),
            (
                MobBehaviorCategory::Communication,
                map.remove(&MobBehaviorCategory::Communication)
                    .unwrap_or_default(),
            ),
            (
                MobBehaviorCategory::Joints,
                map.remove(&MobBehaviorCategory::Joints).unwrap_or_default(),
            ),
        ]
    });

impl MobBehaviorVariant {
    /// Get the category of this behavior variant (reads from strum props).
    pub fn category(&self) -> MobBehaviorCategory {
        match self.get_str("category").unwrap_or("Movement") {
            "Targeting" => MobBehaviorCategory::Targeting,
            "Spawning" => MobBehaviorCategory::Spawning,
            "Timing" => MobBehaviorCategory::Timing,
            "Communication" => MobBehaviorCategory::Communication,
            "Joints" => MobBehaviorCategory::Joints,
            _ => MobBehaviorCategory::Movement,
        }
    }
}

/// Used in behavior trees for attaching several behaviors to a node
#[derive(Component, Clone)]
pub struct MobBehaviorComponent {
    pub behaviors: Vec<MobBehaviorType>,
}

/// Marker component storing the action node name for behavior tree visualization
#[derive(Component, Clone)]
pub struct BehaviorActionName(pub String);

/// Used for transmitting behaviors to other mobs
#[derive(Message)]
pub(crate) struct TransmitBehaviorEvent {
    pub source_entity: Entity,
    pub mob_type: String,
    pub behaviors: Vec<MobBehaviorType>,
}
