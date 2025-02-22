use glam::{Vec3, Vec4};
use std::cell::RefCell;

use crate::physics::constraints::Constraint;

pub enum PhysicsObject {
    RigidBody {
        position: Vec3,
        velocity: Vec3,
        acceleration: Vec3,
        mass: f32,
        bounding_box: Vec4, // x, y, z, size
    },
    DeformableBody {
        positions: Vec<Vec3>,      // Positions of particles
        prev_positions: Vec<Vec3>, // Previous positions of particles
        velocities: Vec<Vec3>,     // Velocities of particles
        masses: Vec<f32>,          // Masses of particles
        bounding_box: Vec4,        // Overall bounding box for broad-phase collision
                                   // Could add indices here to refer to a global particle list, if needed
    },
}

pub struct PhysicsWorld {
    pub objects: Vec<RefCell<PhysicsObject>>,
    pub gravity: Vec3,
    pub constraints: Vec<Box<dyn Constraint>>,
    pub num_iterations: usize,
    pub substeps: usize,
}

impl PhysicsWorld {
    pub fn new(gravity: Vec3) -> Self {
        PhysicsWorld {
            objects: Vec::new(),
            gravity,
            constraints: Vec::new(),
            num_iterations: 10, // Default value, can be tuned
            substeps: 1,
        }
    }

    pub fn add_object(&mut self, object: PhysicsObject) {
        self.objects.push(RefCell::new(object));
    }

    // Add methods to add constraints, e.g.:
    pub fn add_constraint(&mut self, constraint: Box<dyn Constraint>) {
        self.constraints.push(constraint);
    }
}

impl PhysicsWorld {
    pub fn update(&mut self, delta_time: f32) {
        let sub_delta_time = delta_time / self.substeps as f32;
        for _ in 0..self.substeps {
            self.sub_update(sub_delta_time);
        }
    }

    fn sub_update(&mut self, delta_time: f32) {
        let damping = 0.98; // Example damping factor

        for object in &mut self.objects {
            match &mut *object.borrow_mut() {
                PhysicsObject::RigidBody {
                    position,
                    velocity,
                    acceleration,
                    mass,
                    ..
                } => {
                    // Newtonian physics update
                    *velocity += self.gravity * delta_time;
                    *velocity += *acceleration * delta_time;
                    *position += *velocity * delta_time;
                    *acceleration = Vec3::ZERO;
                }
                PhysicsObject::DeformableBody {
                    positions,
                    prev_positions,
                    velocities,
                    masses,
                    ..
                } => {
                    // PBD update for each particle
                    for i in 0..positions.len() {
                        // (1) Store previous position
                        prev_positions[i] = positions[i];

                        // (2) Apply external forces (gravity)
                        velocities[i] += self.gravity * delta_time;

                        // (3) Damp velocities
                        velocities[i] *= damping;

                        // (4) Predict position
                        positions[i] += velocities[i] * delta_time;
                    }
                }
            }
        }

        // (5) Constraint Projection Loop
        for _ in 0..self.num_iterations {
            for constraint in &self.constraints {
                constraint.project(&mut self.objects);
            }
        }

        // (6) Update Velocities and Positions (for DeformableBody)
        for object in &mut self.objects {
            if let PhysicsObject::DeformableBody {
                positions,
                prev_positions,
                velocities,
                ..
            } = &mut *object.borrow_mut()
            {
                for i in 0..positions.len() {
                    velocities[i] = (positions[i] - prev_positions[i]) / delta_time;
                }
            }
        }
    }
}
