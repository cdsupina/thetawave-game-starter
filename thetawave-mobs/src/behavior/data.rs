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

#[derive(Component, Reflect)]
pub(crate) struct BehaviorReceiver(pub Entity);

#[derive(Component)]
pub(crate) struct Target(pub Entity);

#[derive(Clone, PartialEq)]
pub(crate) enum MobBehaviorType {
    MoveDown,
    BrakeHorizontal,
    MoveTo(Vec2),
    FindPlayerTarget,
    MoveToTarget,
    RotateToTarget,
    MoveForward,
    LoseTarget,
    BrakeAngular,
    SpawnMob(Option<Vec<String>>),
    DoForTime(Timer),
    TransmitMobBehavior {
        mob_type: MobType,
        behaviors: Box<Vec<MobBehaviorType>>,
    },
}

#[derive(Component, Clone)]
pub(crate) struct MobBehavior {
    pub behaviors: Vec<MobBehaviorType>,
}

/// Resource containing behavior sequences for mobs
#[derive(Resource)]
pub(crate) struct MobBehaviorsResource {
    pub behaviors: HashMap<MobType, Tree<Behave>>,
}

#[derive(Event)]
pub(crate) struct TransmitBehaviorEvent {
    pub entity: Entity,
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
                            Behave::spawn_named("Movement", MobBehavior { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal]  }),
                        }
                    },
                ),
                (
                    MobType::XhitaraSpitter,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehavior { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal]  }),
                        }
                    },
                ),
                (
                    MobType::XhitaraGyro,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehavior { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal]  }),
                        }
                    },
                ),
                (
                    MobType::XhitaraPacer,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehavior { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal]  }),
                        }
                    },
                ),
                (
                    MobType::FreighterOne,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehavior { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal]  }),

                        }
                    },
                ),
                (
                    MobType::FreighterTwo,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehavior { behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal]  }),

                        }
                    },
                ),
                (
                    MobType::XhitaraMissile,
                    behave! {
                        Behave::Forever => {
                            Behave::Sequence => {
                                Behave::spawn_named("Find Target", MobBehavior{ behaviors: vec![MobBehaviorType::FindPlayerTarget, MobBehaviorType::MoveForward, MobBehaviorType::BrakeAngular]}),
                                Behave::spawn_named("Move To Target", MobBehavior{ behaviors: vec![MobBehaviorType::MoveForward, MobBehaviorType::RotateToTarget, MobBehaviorType::LoseTarget]})
                            }
                        }
                    },
                ),
                (
                    MobType::XhitaraLauncher,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Move and Spawn Missiles", MobBehavior{ behaviors: vec![MobBehaviorType::MoveDown, MobBehaviorType::BrakeHorizontal, MobBehaviorType::SpawnMob(Some(vec!["missiles".to_string()]))]}),
                        }
                    },
                ),
                (
                    MobType::XhitaraTentacleEnd,
                    behave! {
                        Behave::Forever => {
                            Behave::Sequence => {
                                Behave::spawn_named("Find Target", MobBehavior{ behaviors: vec![MobBehaviorType::FindPlayerTarget, MobBehaviorType::BrakeAngular]}),
                                Behave::spawn_named("Move To Target", MobBehavior{ behaviors: vec![MobBehaviorType::MoveForward, MobBehaviorType::RotateToTarget, MobBehaviorType::LoseTarget]})
                            }
                        }
                    },
                ),
                (
                    MobType::XhitaraCyclusk,
                    behave! {
                        Behave::Forever => {
                            Behave::Sequence => {
                                Behave::spawn_named("Find Target", MobBehavior{ behaviors: vec![MobBehaviorType::FindPlayerTarget]}),
                                Behave::spawn_named("Move To Target", MobBehavior{ behaviors: vec![MobBehaviorType::MoveToTarget, MobBehaviorType::RotateToTarget]})
                            }
                        }
                    },
                ),
                (
                    MobType::Trizetheron,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehavior { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(0.0, 50.0))]  }),

                        }
                    },
                ),
                (
                    MobType::Ferritharax,
                    behave! {
                        Behave::Forever => {
                            Behave::Sequence => {
                                Behave::spawn_named("Movement", MobBehavior { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(0.0, 50.0)), MobBehaviorType::DoForTime(Timer::from_seconds(15.0, TimerMode::Once))]  }),
                                Behave::spawn_named("Movement", MobBehavior { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(125.0, 50.0)), MobBehaviorType::DoForTime(Timer::from_seconds(15.0, TimerMode::Once)), MobBehaviorType::TransmitMobBehavior { mob_type: MobType::FerritharaxLeftArm, behaviors: Box::new(vec![MobBehaviorType::MoveDown]) }, MobBehaviorType::TransmitMobBehavior { mob_type: MobType::FerritharaxLeftClaw, behaviors: Box::new(vec![MobBehaviorType::MoveDown]) }, MobBehaviorType::TransmitMobBehavior { mob_type: MobType::FerritharaxLeftShoulder, behaviors: Box::new(vec![MobBehaviorType::MoveDown]) }]  }),
                                Behave::spawn_named("Movement", MobBehavior { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(0.0, 50.0)), MobBehaviorType::DoForTime(Timer::from_seconds(15.0, TimerMode::Once))]  }),
                                Behave::spawn_named("Movement", MobBehavior { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(-125.0, 50.0)), MobBehaviorType::DoForTime(Timer::from_seconds(15.0, TimerMode::Once))]  }),
                            }
                        }
                    },
                ),
            ]),
        }
    }
}
