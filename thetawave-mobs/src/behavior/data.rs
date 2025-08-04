use bevy::{
    ecs::resource::Resource, math::Vec2, platform::collections::HashMap, prelude::Component,
};
use bevy_behave::{Behave, behave, prelude::Tree};

use crate::MobType;

#[derive(Clone)]
pub(crate) enum MoveBehaviorType {
    MoveDown,
    BrakeHorizontal,
    MoveTo(Vec2),
}

#[derive(Component, Clone)]
pub(crate) struct MoveBehavior {
    pub behaviors: Vec<MoveBehaviorType>,
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
                            Behave::spawn_named("Movement", MoveBehavior { behaviors: vec![MoveBehaviorType::MoveDown, MoveBehaviorType::BrakeHorizontal]  }),
                        }
                    },
                ),
                (
                    MobType::XhitaraSpitter,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MoveBehavior { behaviors: vec![MoveBehaviorType::MoveDown, MoveBehaviorType::BrakeHorizontal]  }),
                        }
                    },
                ),
                (
                    MobType::XhitaraGyro,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MoveBehavior { behaviors: vec![MoveBehaviorType::MoveDown, MoveBehaviorType::BrakeHorizontal]  }),
                        }
                    },
                ),
                (
                    MobType::XhitaraPacer,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MoveBehavior { behaviors: vec![MoveBehaviorType::MoveDown, MoveBehaviorType::BrakeHorizontal]  }),
                        }
                    },
                ),
                (
                    MobType::FreighterFront,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MoveBehavior { behaviors: vec![MoveBehaviorType::MoveDown, MoveBehaviorType::BrakeHorizontal]  }),

                        }
                    },
                ),
                (
                    MobType::FreighterOne,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MoveBehavior { behaviors: vec![MoveBehaviorType::MoveDown, MoveBehaviorType::BrakeHorizontal]  }),

                        }
                    },
                ),
                (
                    MobType::FreighterTwo,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MoveBehavior { behaviors: vec![MoveBehaviorType::MoveDown, MoveBehaviorType::BrakeHorizontal]  }),

                        }
                    },
                ),
                (
                    MobType::Trizetheron,
                    behave! {
                        Behave::Forever => {
                            Behave::spawn_named("Movement", MoveBehavior { behaviors: vec![MoveBehaviorType::MoveTo(Vec2::new(0.0, 50.0))]  }),

                        }
                    },
                ),
            ]),
        }
    }
}
