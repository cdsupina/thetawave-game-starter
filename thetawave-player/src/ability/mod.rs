use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        event::Event,
        resource::Resource,
        system::{In, SystemId},
        world::{FromWorld, World},
    },
    platform::collections::HashMap,
};

mod systems;

use crate::{
    PlayerAbility,
    ability::systems::{
        ability_dispatcher_system, fire_blast_ability, fire_bullet_ability, mega_blast_ability,
    },
};

pub struct ThetawaveAbilitiesPlugin {
    pub extended_abilities: HashMap<String, SystemId<In<Entity>>>,
}

impl Plugin for ThetawaveAbilitiesPlugin {
    fn build(&self, app: &mut App) {
        let ability_registry = AbilityRegistry::from_world(app.world_mut())
            .with_extended_abilities(self.extended_abilities.clone());

        app.insert_resource(ability_registry)
            .add_systems(Update, ability_dispatcher_system);
    }
}

#[derive(Event, Debug)]
pub struct ExecutePlayerAbilityEvent {
    pub ability_type: String,
    pub player_entity: Entity,
}

#[derive(Component)]
pub struct EquippedAbilities {
    pub abilities: HashMap<PlayerAbility, String>,
}

#[derive(Resource, Debug)]
struct AbilityRegistry {
    abilities: HashMap<String, SystemId<In<Entity>>>,
}

impl FromWorld for AbilityRegistry {
    fn from_world(world: &mut World) -> Self {
        let mut abilities = HashMap::new();

        abilities.insert(
            "fire_blast".to_string(),
            world.register_system(fire_blast_ability),
        );

        abilities.insert(
            "mega_blast".to_string(),
            world.register_system(mega_blast_ability),
        );

        abilities.insert(
            "fire_bullet".to_string(),
            world.register_system(fire_bullet_ability),
        );

        AbilityRegistry { abilities }
    }
}

impl AbilityRegistry {
    fn with_extended_abilities(mut self, abilities: HashMap<String, SystemId<In<Entity>>>) -> Self {
        self.abilities.extend(abilities);

        self
    }
}
