use super::component::Component;
use super::entity::{Entity, EntityBuilder, EntityId};
use std::collections::HashMap;

pub struct World {
    next_entity_id: usize,
    entities: Vec<Entity>,
    entities_to_remove: Vec<EntityId>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            entities: Vec::new(),
            entities_to_remove: Vec::new(),
        }
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
        EntityBuilder::new()
    }

    pub fn spawn(&mut self, builder: EntityBuilder) -> EntityId {
        let id = EntityId(self.next_entity_id);
        self.next_entity_id += 1;
        let entity = builder.build(id);
        self.entities.push(entity);
        id
    }

    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id() == id)
    }

    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id() == id)
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn entities_mut(&mut self) -> &mut [Entity] {
        &mut self.entities
    }

    pub fn remove_entity(&mut self, id: EntityId) {
        self.entities_to_remove.push(id);
    }

    pub fn query(&self) -> super::entity::Query {
        super::entity::Query::new()
    }

    pub fn maintain(&mut self) {
        // Remove marked entities
        if !self.entities_to_remove.is_empty() {
            let remove_set: std::collections::HashSet<_> =
                self.entities_to_remove.drain(..).collect();
            self.entities.retain(|e| !remove_set.contains(&e.id()));
        }
    }

    // Helper method to get all entities with a specific component
    pub fn get_with_component<T: Component>(&self) -> Vec<&Entity> {
        self.entities
            .iter()
            .filter(|e| e.has_component::<T>())
            .collect()
    }

    // Helper method to get all entities with multiple components
    pub fn get_with_components<T1: Component, T2: Component>(&self) -> Vec<&Entity> {
        self.entities
            .iter()
            .filter(|e| e.has_component::<T1>() && e.has_component::<T2>())
            .collect()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

// Helper struct for tracking component changes
#[derive(Default)]
pub struct WorldDiff {
    added_components: HashMap<EntityId, Vec<std::any::TypeId>>,
    removed_components: HashMap<EntityId, Vec<std::any::TypeId>>,
    modified_components: HashMap<EntityId, Vec<std::any::TypeId>>,
}

impl WorldDiff {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track_component_add<T: Component>(&mut self, entity: EntityId) {
        self.added_components
            .entry(entity)
            .or_default()
            .push(T::component_type());
    }

    pub fn track_component_remove<T: Component>(&mut self, entity: EntityId) {
        self.removed_components
            .entry(entity)
            .or_default()
            .push(T::component_type());
    }

    pub fn track_component_modify<T: Component>(&mut self, entity: EntityId) {
        self.modified_components
            .entry(entity)
            .or_default()
            .push(T::component_type());
    }
}
