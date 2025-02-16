mod component;
mod entity;
pub mod scene_graph;
mod system;
mod world;

pub use component::{Component, ComponentStorage};
pub use entity::{Entity, EntityBuilder};
pub use scene_graph::{Node, RenderObject, Transform};
pub use system::System;
pub use world::World;

use crate::graphics::renderer::RenderContext;
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

    pub fn render(&self, graphics: &mut RenderContext) -> Result<(), crate::core::error::Error> {
        // TODO: Implement scene rendering using scene graph
        Ok(())
    }

    pub fn create_entity(&mut self) -> entity::EntityBuilder {
        self.world.create_entity()
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
