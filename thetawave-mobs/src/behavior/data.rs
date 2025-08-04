use bevy::{ecs::resource::Resource, platform::collections::HashMap, prelude::Component};
use bevy_behave::{Behave, behave, prelude::Tree};

use crate::MobType;

#[derive(Clone)]
pub(crate) enum MoveBehaviorType {
    MoveDown,
    BrakeHorizontal,
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
            ]),
        }
    }
}
