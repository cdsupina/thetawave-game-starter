use bevy::{
    ecs::{entity::Entity, event::Event, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
    prelude::Component,
    reflect::Reflect,
    time::Timer,
};
use bevy_behave::{prelude::Tree, Behave};

/// Used for receiving behaviors from another mob's TransmitMobBehavior
/// The entity is the mob entity that behaviors can be receieved from
#[derive(Component, Reflect)]
pub(crate) struct BehaviorReceiverComponent(pub Entity);

/// Target component used for behaviors that target other entities
/// Such as homing missiles
#[derive(Component, Reflect)]
pub(crate) struct TargetComponent(pub Entity);

/// Mob behaviors that can be run together at a single node in the behavior tree
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum MobBehaviorType {
    // Movement behaviors
    MoveDown,
    #[allow(dead_code)] // Available for future use in behavior trees
    MoveUp,
    MoveLeft,
    MoveRight,
    BrakeHorizontal,
    BrakeAngular,
    
    /// Move to specific position
    MoveTo(Vec2),
    
    // Targeting behaviors
    FindPlayerTarget,
    MoveToTarget,
    RotateToTarget,
    MoveForward,
    LoseTarget,
    
    // Spawning behaviors
    SpawnMob { 
        keys: Option<Vec<String>>
    },
    SpawnProjectile { 
        keys: Option<Vec<String>>
    },
    
    // Timing behaviors
    DoForTime(Timer),
    
    // Communication behaviors  
    TransmitMobBehavior {
        mob_type: &'static str,
        behaviors: Vec<MobBehaviorType>,
    },
    
    // Joint behaviors (for future use)
    #[allow(dead_code)]
    RotateJointsClockwise { 
        keys: Vec<String>
    },
}

/// Used in behavior trees for attaching several behaviors to a node
#[derive(Component, Clone)]
pub(crate) struct MobBehaviorComponent {
    pub behaviors: Vec<MobBehaviorType>,
}

/// Resource for mapping behavior trees to mob types
/// Used for mob spawning mobs
#[derive(Resource)]
pub(crate) struct MobBehaviorsResource {
    pub behaviors: HashMap<String, Tree<Behave>>,
}

/// Used for transmitting behaviors to other mobs
#[derive(Event)]
pub(crate) struct TransmitBehaviorEvent {
    pub source_entity: Entity,
    pub mob_type: &'static str,
    pub behaviors: Vec<MobBehaviorType>,
}

// Behaviors are now loaded from mob_behaviors.toml via the plugin
