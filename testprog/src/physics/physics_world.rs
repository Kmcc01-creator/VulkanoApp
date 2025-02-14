use glam::{Quat, Vec3};
use std::collections::HashMap;

use super::collider::{Collider, AABB};
use super::rigidbody::{BodyType, RigidBody};
use crate::core::error::Error;

pub struct Contact {
    pub body_a: usize,
    pub body_b: usize,
    pub normal: Vec3,
    pub penetration: f32,
    pub contact_point: Vec3,
}

#[derive(Default)]
pub struct PhysicsWorld {
    rigid_bodies: HashMap<usize, RigidBody>,
    colliders: HashMap<usize, Collider>,
    body_collider_map: HashMap<usize, usize>,
    gravity: Vec3,
    contacts: Vec<Contact>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            rigid_bodies: HashMap::new(),
            colliders: HashMap::new(),
            body_collider_map: HashMap::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            contacts: Vec::new(),
        }
    }

    pub fn add_rigidbody(&mut self, body: RigidBody) -> usize {
        let id = self.next_body_id();
        self.rigid_bodies.insert(id, body);
        id
    }

    pub fn add_collider(&mut self, collider: Collider, body_id: usize) {
        let collider_id = self.next_collider_id();
        self.colliders.insert(collider_id, collider);
        self.body_collider_map.insert(body_id, collider_id);
    }

    pub fn step(&mut self, dt: f32, gravity: Vec3, iterations: u32) {
        // Clear previous contacts
        self.contacts.clear();

        // Broad phase collision detection
        let mut potential_contacts = self.broad_phase();

        // Narrow phase collision detection
        self.narrow_phase(&mut potential_contacts);

        // Solve constraints
        for _ in 0..iterations {
            self.solve_contacts(dt);
        }

        // Integrate velocities
        self.integrate(dt, gravity);
    }

    fn next_body_id(&self) -> usize {
        self.rigid_bodies.keys().max().map_or(0, |&id| id + 1)
    }

    fn next_collider_id(&self) -> usize {
        self.colliders.keys().max().map_or(0, |&id| id + 1)
    }

    fn broad_phase(&self) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        let bodies: Vec<_> = self.rigid_bodies.keys().collect();

        for (i, &body_a) in bodies.iter().enumerate() {
            if let Some(collider_a) = self.get_body_collider(body_a) {
                let aabb_a = self.compute_aabb(body_a, collider_a);

                for &body_b in &bodies[i + 1..] {
                    if let Some(collider_b) = self.get_body_collider(body_b) {
                        let aabb_b = self.compute_aabb(body_b, collider_b);

                        if aabb_a.intersects(&aabb_b) {
                            pairs.push((body_a, body_b));
                        }
                    }
                }
            }
        }

        pairs
    }

    fn narrow_phase(&mut self, pairs: &[(usize, usize)]) {
        for &(body_a, body_b) in pairs {
            if let (Some(rb_a), Some(rb_b)) = (
                self.rigid_bodies.get(&body_a),
                self.rigid_bodies.get(&body_b),
            ) {
                // Skip if both bodies are static or kinematic
                if rb_a.body_type == BodyType::Static && rb_b.body_type == BodyType::Static {
                    continue;
                }

                if let (Some(collider_a), Some(collider_b)) = (
                    self.get_body_collider(body_a),
                    self.get_body_collider(body_b),
                ) {
                    // TODO: Implement detailed collision detection based on shape types
                    // For now, we'll just use AABB intersection
                    let aabb_a = self.compute_aabb(body_a, collider_a);
                    let aabb_b = self.compute_aabb(body_b, collider_b);

                    if aabb_a.intersects(&aabb_b) {
                        // Create a basic contact
                        let normal = (rb_b.position - rb_a.position).normalize();
                        self.contacts.push(Contact {
                            body_a,
                            body_b,
                            normal,
                            penetration: 0.1, // placeholder
                            contact_point: rb_a.position + normal * 0.5,
                        });
                    }
                }
            }
        }
    }

    fn solve_contacts(&mut self, dt: f32) {
        for contact in &self.contacts {
            if let (Some(body_a), Some(body_b)) = (
                self.rigid_bodies.get_mut(&contact.body_a),
                self.rigid_bodies.get_mut(&contact.body_b),
            ) {
                // Basic impulse resolution
                let relative_velocity = body_b.linear_velocity - body_a.linear_velocity;
                let velocity_along_normal = relative_velocity.dot(contact.normal);

                // Don't resolve if objects are separating
                if velocity_along_normal > 0.0 {
                    continue;
                }

                let restitution = 0.5; // Could be derived from materials
                let j = -(1.0 + restitution) * velocity_along_normal;
                let impulse = contact.normal * j;

                if body_a.body_type == BodyType::Dynamic {
                    body_a.linear_velocity -= impulse / body_a.mass;
                }
                if body_b.body_type == BodyType::Dynamic {
                    body_b.linear_velocity += impulse / body_b.mass;
                }
            }
        }
    }

    fn integrate(&mut self, dt: f32, gravity: Vec3) {
        for body in self.rigid_bodies.values_mut() {
            if body.body_type != BodyType::Dynamic || body.sleeping {
                continue;
            }

            // Apply gravity
            body.linear_velocity += gravity * body.gravity_scale * dt;

            // Apply damping
            body.linear_velocity *= 1.0 / (1.0 + body.linear_damping * dt);
            body.angular_velocity *= 1.0 / (1.0 + body.angular_damping * dt);

            // Update position and rotation
            body.position += body.linear_velocity * dt;
            let rotation_vec = body.angular_velocity * dt;
            if rotation_vec.length_squared() > 0.0 {
                let rotation = Quat::from_rotation_vec(rotation_vec);
                body.rotation = rotation * body.rotation;
            }
        }
    }

    fn get_body_collider(&self, body_id: usize) -> Option<&Collider> {
        self.body_collider_map
            .get(&body_id)
            .and_then(|&collider_id| self.colliders.get(&collider_id))
    }

    fn compute_aabb(&self, body_id: usize, collider: &Collider) -> AABB {
        let body = &self.rigid_bodies[&body_id];
        collider.compute_aabb(body.position, body.rotation)
    }
}
