mod collider;
mod physics_world;
mod rigidbody;

pub use collider::{Collider, ColliderType};
pub use physics_world::PhysicsWorld;
pub use rigidbody::RigidBody;

use glam::{Vec2, Vec3};

/// Physics configuration options
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    pub gravity: Vec3,
    pub timestep: f32,
    pub iterations: u32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            timestep: 1.0 / 60.0,
            iterations: 8,
        }
    }
}

pub struct PhysicsEngine {
    config: PhysicsConfig,
    world: PhysicsWorld,
}

impl PhysicsEngine {
    pub fn new(config: PhysicsConfig) -> Self {
        Self {
            config,
            world: PhysicsWorld::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        let step_count = (delta_time / self.config.timestep).ceil() as u32;
        for _ in 0..step_count {
            self.step();
        }
    }

    fn step(&mut self) {
        // Update physics simulation
        self.world.step(self.config.timestep);
    }

    pub fn add_rigidbody(&mut self, rigidbody: RigidBody) -> usize {
        self.world.add_body(rigidbody)
    }

    pub fn add_collider(&mut self, collider: Collider, rigidbody_handle: usize) {
        self.world.set_body_collider(rigidbody_handle, collider);
    }
}

impl Default for PhysicsEngine {
    fn default() -> Self {
        Self::new(PhysicsConfig::default())
    }
}
