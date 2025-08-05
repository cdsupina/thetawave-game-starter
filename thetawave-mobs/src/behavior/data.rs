use bevy::{
    ecs::{entity::Entity, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
    prelude::Component,
};
use bevy_behave::{Behave, behave, prelude::Tree};

use crate::MobType;

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
                    MobType::FreighterFront,
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
                    MobType::Trizetheron,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MobBehavior { behaviors: vec![MobBehaviorType::MoveTo(Vec2::new(0.0, 50.0))]  }),

                        }
                    },
                ),
                (
                    MobType::XhitaraMissile,
                    behave! {
                        Behave::Forever => {
                            Behave::Sequence => {
                                Behave::spawn_named("Find Target", MobBehavior{ behaviors: vec![MobBehaviorType::FindPlayerTarget]}),
                                Behave::spawn_named("Move To Target", MobBehavior{ behaviors: vec![MobBehaviorType::MoveForward, MobBehaviorType::RotateToTarget]})
                            }
                        }
                    },
                ),
            ]),
        }
    }
}
