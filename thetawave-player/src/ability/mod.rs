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
    platform::collections::{HashMap, HashSet},
};

mod systems;

use crate::{
    PlayerAbility,
    ability::systems::{
        ability_dispatcher_system, charge_ability, charge_ability_timer_system, fire_blast_ability, fire_bullet_ability,
        mega_blast_ability,
    },
};

pub struct ThetawaveAbilitiesPlugin {
    pub extended_abilities: HashMap<String, SystemId<In<Entity>>>,
    pub extended_duration_abilities: HashSet<String>,
}

impl Plugin for ThetawaveAbilitiesPlugin {
    fn build(&self, app: &mut App) {
        let ability_registry = AbilityRegistry::from_world(app.world_mut())
            .with_extended_abilities(self.extended_abilities.clone(), self.extended_duration_abilities.clone());

        app.insert_resource(ability_registry)
            .add_systems(Update, (ability_dispatcher_system, charge_ability_timer_system));
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
pub struct AbilityRegistry {
    pub abilities: HashMap<String, SystemId<In<Entity>>>,
    pub duration_abilities: HashSet<String>,
}

impl FromWorld for AbilityRegistry {
    fn from_world(world: &mut World) -> Self {
        let mut abilities = HashMap::new();

        abilities.insert(
            "fire_blast".to_string(),
            world.register_system(fire_blast_ability),
        );

        abilities.insert(
            "fire_bullet".to_string(),
            world.register_system(fire_bullet_ability),
        );

        abilities.insert(
            "mega_blast".to_string(),
            world.register_system(mega_blast_ability),
        );

        abilities.insert("charge".to_string(), world.register_system(charge_ability));

        let mut duration_abilities = HashSet::new();
        duration_abilities.insert("charge".to_string());

        AbilityRegistry { abilities, duration_abilities }
    }
}

impl AbilityRegistry {
    fn with_extended_abilities(mut self, abilities: HashMap<String, SystemId<In<Entity>>>, duration_abilities: HashSet<String>) -> Self {
        self.abilities.extend(abilities);
        self.duration_abilities.extend(duration_abilities);

        self
    }
}
