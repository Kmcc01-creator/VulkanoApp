mod component;
mod entity;
mod system;
mod world;

pub use component::{Component, ComponentStorage};
pub use entity::{Entity, EntityBuilder};
pub use system::System;
pub use world::World;

use crate::graphics::Graphics;
use crate::resource::ResourceManager;

pub struct SceneManager {
    world: World,
    systems: Vec<Box<dyn System>>,
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            systems: Vec::new(),
        }
    }

    pub fn update(&mut self, resources: &ResourceManager) {
        for system in &mut self.systems {
            system.update(&mut self.world, resources);
        }
    }

    pub fn render(&self, graphics: &mut Graphics) -> Result<(), crate::core::error::Error> {
        // TODO: Implement scene rendering
        Ok(())
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
        EntityBuilder::new(&mut self.world)
    }

    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}
