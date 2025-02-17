use super::component::{Component, ComponentStorage};
use std::any::TypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub(crate) usize);

pub struct Entity {
    id: EntityId,
    components: ComponentStorage,
}

impl Entity {
    pub(crate) fn new(id: EntityId) -> Self {
        Self {
            id,
            components: ComponentStorage::new(),
        }
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn add_component<T: Component>(&mut self, component: T) {
        self.components.add(component);
    }

    pub fn get_component<T: Component>(&self) -> Option<&T> {
        self.components.get::<T>()
    }

    pub fn get_component_mut<T: Component>(&mut self) -> Option<&mut T> {
        self.components.get_mut::<T>()
    }

    pub fn has_component<T: Component>(&self) -> bool {
        self.components.has::<T>()
    }

    pub fn remove_component<T: Component>(&mut self) -> Option<Box<T>> {
        self.components.remove::<T>()
    }
}

pub struct EntityBuilder {
    components: ComponentStorage,
}

impl EntityBuilder {
    pub(crate) fn new() -> Self {
        Self {
            components: ComponentStorage::new(),
        }
    }

    pub fn with_component<T: Component>(mut self, component: T) -> Self {
        self.components.add(component);
        self
    }

    pub(crate) fn build(self, id: EntityId) -> Entity {
        Entity {
            id,
            components: self.components,
        }
    }
}

// Entity query system for filtering entities based on components
pub struct Query<'a> {
    required_components: Vec<TypeId>,
    entities: Vec<&'a Entity>,
}

impl<'a> Query<'a> {
    pub fn new() -> Self {
        Self {
            required_components: Vec::new(),
            entities: Vec::new(),
        }
    }

    pub fn with_component<T: Component>(mut self) -> Self {
        self.required_components.push(T::component_type());
        self
    }

    pub fn matches(&self, entity: &Entity) -> bool {
        self.required_components
            .iter()
            .all(|&type_id| entity.components.has_type(type_id))
    }

    pub fn collect(mut self, entities: &'a [Entity]) -> Vec<&'a Entity> {
        self.entities = entities.iter().filter(|e| self.matches(e)).collect();
        self.entities
    }
}

impl<'a> Default for Query<'a> {
    fn default() -> Self {
        Self::new()
    }
}
