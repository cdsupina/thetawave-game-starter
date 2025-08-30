use bevy::{
    ecs::{entity::Entity, event::Event, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
    prelude::Component,
    reflect::Reflect,
    time::{Timer, TimerMode},
};
use bevy_behave::{Behave, behave, prelude::Tree};

use crate::MobType;

/// Used for receiving behaviors from another mob's TransmitMobBehavior
/// The entity is the mob entity that behaviors can be receieved from
#[derive(Component, Reflect)]
pub(crate) struct BehaviorReceiverComponent(pub Entity);

/// Target component used for behaviors that target other entities
/// Such as homing missiles
#[derive(Component, Reflect)]
pub(crate) struct TargetComponent(pub Entity);

/// Mob behaviors that can be run together at a single node in the behavior tree
#[derive(Clone, PartialEq)]
pub(crate) enum MobBehaviorType {
    MoveDown,
    #[allow(dead_code)] // Available for future use in behavior trees
    MoveUp,
    MoveLeft,
    MoveRight,
    BrakeHorizontal,
    MoveTo(Vec2),
    FindPlayerTarget,
    MoveToTarget,
    RotateToTarget,
    MoveForward,
    LoseTarget,
    BrakeAngular,
    SpawnMob(Option<Vec<String>>),
    SpawnProjectile(Option<Vec<String>>),
    DoForTime(Timer),
    TransmitMobBehavior {
        mob_type: MobType,
        behaviors: Vec<MobBehaviorType>,
    },
    #[allow(dead_code)]
    RotateJointsClockwise(Vec<String>),
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
    pub behaviors: HashMap<MobType, Tree<Behave>>,
}

/// Used for transmitting behaviors to other mobs
#[derive(Event)]
pub(crate) struct TransmitBehaviorEvent {
    pub source_entity: Entity,
    pub mob_type: MobType,
    pub behaviors: Vec<MobBehaviorType>,
}

impl MobBehaviorsResource {
    pub fn new() -> Self {
        Self {
            behaviors: HashMap::from([
                (
                    MobType::XhitaraGrunt,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal]  }),
                        }
                    },
                ),
                (
                    MobType::XhitaraSpitter,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal, MobBehaviorType::SpawnProjectile(Some(vec!["south".to_string()]))]  }),
                        }
                    },
                ),
                (
                    MobType::XhitaraGyro,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal, MobBehaviorType::SpawnProjectile(Some(vec!["south".to_string(), "east".to_string(), "west".to_string()]))]  }),
                        }
                    },
                ),
                (
                    MobType::XhitaraPacer,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal, MobBehaviorType::SpawnProjectile(Some(vec!["south".to_string()]))]  }),
                        }
                    },
                ),
                (
                    MobType::FreighterOne,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal]  }),

                        }
                    },
                ),
                (
                    MobType::FreighterTwo,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal]  }),

                        }
                    },
                ),
                (
                    MobType::XhitaraMissile,
                    behave! {
                        Behave::Forever => {
                            Behave::Sequence => {
                                Behave::spawn_named("Find Target", MobBehaviorComponent{ behaviors: vec![MobBehaviorType::FindPlayerTarget, MobBehaviorType::MoveForward, MobBehaviorType::BrakeAngular]}),
                                Behave::spawn_named("Move To Target", MobBehaviorComponent{ behaviors: vec![MobBehaviorType::MoveForward, MobBehaviorType::RotateToTarget, MobBehaviorType::LoseTarget]})
                            }
                        }
                    },
                ),
                (
                    MobType::XhitaraLauncher,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Move and Spawn Missiles", MobBehaviorComponent{ behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal, MobBehaviorType::SpawnMob(Some(vec!["missiles".to_string()]))]}),
                        }
                    },
                ),
                (
                    MobType::XhitaraTentacleEnd,
                    behave! {
                        Behave::Forever => {
                            Behave::Sequence => {
                                Behave::spawn_named("Find Target", MobBehaviorComponent{ behaviors: vec![MobBehaviorType::FindPlayerTarget, MobBehaviorType::BrakeAngular]}),
                                Behave::spawn_named("Move To Target", MobBehaviorComponent{ behaviors: vec![MobBehaviorType::MoveForward, MobBehaviorType::RotateToTarget, MobBehaviorType::LoseTarget]})
                            }
                        }
                    },
                ),
                (
                    MobType::XhitaraCyclusk,
                    behave! {
                        Behave::Forever => {
                            Behave::Sequence => {
                                Behave::spawn_named("Find Target", MobBehaviorComponent{ behaviors: vec![MobBehaviorType::FindPlayerTarget]}),
                                Behave::spawn_named("Move To Target", MobBehaviorComponent{ behaviors: vec![MobBehaviorType::MoveToTarget, MobBehaviorType::RotateToTarget]})
                            }
                        }
                    },
                ),
                (
                    MobType::Trizetheron,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(0.0, 50.0))]  }),

                        }
                    },
                ),
                (
                    MobType::Ferritharax,
                    behave! {
                        Behave::Forever => {
                            Behave::Sequence => {
                                Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(0.0, 50.0)), MobBehaviorType::DoForTime(Timer::from_seconds(15.0, TimerMode::Once))]  }),
                                Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(125.0, 50.0)), MobBehaviorType::DoForTime(Timer::from_seconds(15.0, TimerMode::Once)),  MobBehaviorType::TransmitMobBehavior { mob_type: MobType::FerritharaxLeftClaw, behaviors: vec![MobBehaviorType::MoveRight] }, MobBehaviorType::TransmitMobBehavior { mob_type: MobType::FerritharaxRightClaw, behaviors: vec![MobBehaviorType::MoveLeft] }]  }),
                                Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(0.0, 50.0)), MobBehaviorType::DoForTime(Timer::from_seconds(15.0, TimerMode::Once))]  }),
                                Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(-125.0, 50.0)), MobBehaviorType::DoForTime(Timer::from_seconds(15.0, TimerMode::Once))]  }),
                            }
                        }
                    },
                ),
            ]),
        }
    }
}
