use bevy::{
    ecs::{entity::Entity, event::Event, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
    prelude::Component,
    reflect::Reflect,
    time::{Timer, TimerMode},
};
use bevy_behave::{Behave, behave, prelude::Tree};

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
        mob_type: &'static str,
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
    pub behaviors: HashMap<String, Tree<Behave>>,
}

/// Used for transmitting behaviors to other mobs
#[derive(Event)]
pub(crate) struct TransmitBehaviorEvent {
    pub source_entity: Entity,
    pub mob_type: &'static str,
    pub behaviors: Vec<MobBehaviorType>,
}

impl MobBehaviorsResource {
    pub fn new() -> Self {
        Self {
            behaviors: HashMap::from([
                (
                    "xhitara_grunt_mob".to_string(),
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal]  }),
                        }
                    },
                ),
                (
                    "xhitara_spitter_mob".to_string(),
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal, MobBehaviorType::SpawnProjectile(Some(vec!["south".to_string()]))]  }),
                        }
                    },
                ),
                (
                    "xhitara_gyro_mob".to_string(),
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal, MobBehaviorType::SpawnProjectile(Some(vec!["south".to_string(), "east".to_string(), "west".to_string()]))]  }),
                        }
                    },
                ),
                (
                    "xhitara_pacer_mob".to_string(),
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal, MobBehaviorType::SpawnProjectile(Some(vec!["south".to_string()]))]  }),
                        }
                    },
                ),
                (
                    "freighter_one_mob".to_string(),
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal]  }),

                        }
                    },
                ),
                (
                    "freighter_two_mob".to_string(),
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal]  }),

                        }
                    },
                ),
                (
                    "xhitara_missile_mob".to_string(),
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
                    "xhitara_launcher_mob".to_string(),
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Move and Spawn Missiles", MobBehaviorComponent{ behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal, MobBehaviorType::SpawnMob(Some(vec!["missiles".to_string()]))]}),
                        }
                    },
                ),
                (
                    "xhitara_tentacle_end_mob".to_string(),
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
                    "xhitara_cyclusk_mob".to_string(),
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
                    "trizetheron_mob".to_string(),
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(0.0, 50.0))]  }),

                        }
                    },
                ),
                (
                    "ferritharax_head_mob".to_string(),
                    behave! {
                        Behave::Forever => {
                            Behave::Sequence => {
                                Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(0.0, 50.0)), MobBehaviorType::DoForTime(Timer::from_seconds(15.0, TimerMode::Once))]  }),
                                Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(125.0, 50.0)), MobBehaviorType::DoForTime(Timer::from_seconds(15.0, TimerMode::Once)),  MobBehaviorType::TransmitMobBehavior { mob_type: "ferritharax_left_claw_mob", behaviors: vec![MobBehaviorType::MoveRight] }, MobBehaviorType::TransmitMobBehavior { mob_type: "ferritharax_right_claw_mob", behaviors: vec![MobBehaviorType::MoveLeft] }]  }),
                                Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(0.0, 50.0)), MobBehaviorType::DoForTime(Timer::from_seconds(15.0, TimerMode::Once))]  }),
                                Behave::spawn_named("Movement", MobBehaviorComponent { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(-125.0, 50.0)), MobBehaviorType::DoForTime(Timer::from_seconds(15.0, TimerMode::Once)), MobBehaviorType::TransmitMobBehavior { mob_type: "ferritharax_left_claw_mob", behaviors: vec![MobBehaviorType::MoveRight] }, MobBehaviorType::TransmitMobBehavior { mob_type: "ferritharax_right_claw_mob", behaviors: vec![MobBehaviorType::MoveLeft] }]  }),
                            }
                        }
                    },
                ),
            ]),
        }
    }
}
