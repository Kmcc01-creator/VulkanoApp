use glam::{Quat, Vec3};
use std::collections::HashMap;

use super::collider::{Collider, AABB};
use super::rigidbody::{BodyType, RigidBody};

#[derive(Debug)]
pub struct Contact {
    pub normal: Vec3,
    pub penetration: f32,
    pub position: Vec3,
}

#[derive(Debug)]
pub struct CollisionPair {
    body_a: usize,
    body_b: usize,
    contact: Contact,
}

#[derive(Debug)]
pub struct PhysicsWorld {
    bodies: Vec<RigidBody>,
    colliders: Vec<Option<Collider>>,
    gravity: Vec3,
    iterations: usize,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            bodies: Vec::new(),
            colliders: Vec::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            iterations: 4,
        }
    }
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self::default()
    }

    fn detect_collisions(&self) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        let body_count = self.bodies.len();

        for i in 0..body_count {
            if let Some(collider_a) = self.get_body_collider(i) {
                let body_a = &self.bodies[i];
                let aabb_a = collider_a.compute_aabb(body_a.position, body_a.rotation);

                for j in (i + 1)..body_count {
                    if let Some(collider_b) = self.get_body_collider(j) {
                        let body_b = &self.bodies[j];
                        let aabb_b = collider_b.compute_aabb(body_b.position, body_b.rotation);
                        if aabb_a.intersects(&aabb_b) {
                            pairs.push((i, j));
                        }
                    }
                }
            }
        }

        pairs
    }

    pub fn step(&mut self, dt: f32) {
        // Integrate forces
        for body in &mut self.bodies {
            if body.body_type == BodyType::Static {
                continue;
            }

            // Apply gravity
            body.force += body.gravity_scale * self.gravity * body.mass;

            // Semi-implicit Euler integration
            body.velocity += body.force / body.mass * dt;
            body.angular_velocity += body.torque / body.inertia * dt;

            // Update position and rotation
            body.position += body.velocity * dt;
            if body.angular_velocity.length_squared() > 1e-6 {
                let rotation_vec = body.angular_velocity * dt;
                let rotation =
                    Quat::from_axis_angle(rotation_vec.normalize(), rotation_vec.length());
                body.rotation = rotation * body.rotation;
            }

            // Clear forces
            body.force = Vec3::ZERO;
            body.torque = Vec3::ZERO;
        }

        // Collision detection
        let collision_pairs = self.detect_collisions();

        // Collision resolution iterations
        for _ in 0..self.iterations {
            for &(i, j) in &collision_pairs {
                // Get collision data
                let (pos_a, pos_b, vel_a, vel_b, mass_a, mass_b, rest_a, rest_b, type_a, type_b) = {
                    let body_a = &self.bodies[i];
                    let body_b = &self.bodies[j];
                    (
                        body_a.position,
                        body_b.position,
                        body_a.velocity,
                        body_b.velocity,
                        body_a.mass,
                        body_b.mass,
                        body_a.restitution,
                        body_b.restitution,
                        body_a.body_type,
                        body_b.body_type,
                    )
                };

                // Skip static-static collisions
                if type_a == BodyType::Static && type_b == BodyType::Static {
                    continue;
                }

                // Create basic contact (this would normally come from collision detection)
                let contact = Contact {
                    normal: (pos_b - pos_a).normalize(),
                    penetration: 0.01, // Example value
                    position: (pos_a + pos_b) * 0.5,
                };

                // Calculate impulse
                let rel_vel = vel_b - vel_a;
                let restitution = (rest_a + rest_b) * 0.5;
                let j = -(1.0 + restitution) * Vec3::dot(rel_vel, contact.normal);
                let impulse = contact.normal * j;

                // Apply impulse and position correction
                let percent = 0.2;
                let slop = 0.01;
                let correction = contact.normal * (contact.penetration - slop).max(0.0) * percent
                    / (1.0 / mass_a + 1.0 / mass_b);

                // Update bodies
                if type_a == BodyType::Dynamic {
                    let body = &mut self.bodies[i];
                    body.velocity -= impulse / mass_a;
                    body.position -= correction / mass_a;
                }

                if type_b == BodyType::Dynamic {
                    let body = &mut self.bodies[j];
                    body.velocity += impulse / mass_b;
                    body.position += correction / mass_b;
                }
            }
        }

        // Update sleeping state
        for body in &mut self.bodies {
            if body.body_type != BodyType::Dynamic || body.sleeping {
                continue;
            }

            if body.velocity.length_squared() < 0.01
                && body.angular_velocity.length_squared() < 0.01
            {
                body.set_sleeping(true);
            }
        }
    }

    pub fn add_body(&mut self, body: RigidBody) -> usize {
        let index = self.bodies.len();
        self.bodies.push(body);
        self.colliders.push(None);
        index
    }

    pub fn set_body_collider(&mut self, body_id: usize, collider: Collider) {
        if body_id < self.colliders.len() {
            self.colliders[body_id] = Some(collider);
        }
    }

    fn get_body_collider(&self, body_id: usize) -> Option<&Collider> {
        self.colliders.get(body_id).and_then(|c| c.as_ref())
    }
}
