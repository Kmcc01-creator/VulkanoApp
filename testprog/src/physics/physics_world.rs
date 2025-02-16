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

    fn integrate_forces(&mut self, dt: f32) {
        self.bodies.iter_mut().for_each(|body| {
            let body_type = body.body_type;
            if let BodyType::Dynamic = body_type {
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
        });
    }

    fn detect_collisions(&self) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        let body_count = self.bodies.len();

        for i in 0..body_count {
            if let Some(collider_a) = self.get_body_collider(i) {
                if let Some(body_a) = self.bodies.get(i) {
                    let aabb_a = collider_a.compute_aabb(body_a.position, body_a.rotation);

                    for j in (i + 1)..body_count {
                        if let Some(collider_b) = self.get_body_collider(j) {
                            if let Some(body_b) = self.bodies.get(j) {
                                let aabb_b =
                                    collider_b.compute_aabb(body_b.position, body_b.rotation);
                                if aabb_a.intersects(&aabb_b) {
                                    pairs.push((i, j));
                                }
                            }
                        }
                    }
                }
            }
        }

        pairs
    }

    fn compute_collision_response(
        &self,
        body_a: &RigidBody,
        body_b: &RigidBody,
    ) -> Option<(Vec3, Vec3)> {
        if body_a.body_type == BodyType::Static && body_b.body_type == BodyType::Static {
            return None;
        }

        let normal = (body_b.position - body_a.position).normalize();
        let rel_vel = body_b.velocity - body_a.velocity;
        let restitution = (body_a.restitution + body_b.restitution) * 0.5;

        let penetration = 0.01; // Example value
        let j = -(1.0 + restitution) * Vec3::dot(rel_vel, normal);
        let impulse = normal * j;

        Some((impulse, normal * penetration))
    }

    pub fn step(&mut self, dt: f32) {
        // Integrate forces
        self.integrate_forces(dt);

        // Collision detection and resolution
        let collision_pairs = self.detect_collisions();

        // Collision resolution iterations
        for _ in 0..self.iterations {
            let mut responses = Vec::new();

            // Gather collision responses
            for &(i, j) in &collision_pairs {
                if let (Some(body_a), Some(body_b)) = (self.bodies.get(i), self.bodies.get(j)) {
                    if let Some((impulse, correction)) =
                        self.compute_collision_response(body_a, body_b)
                    {
                        responses.push((i, j, impulse, correction));
                    }
                }
            }

            // Apply responses
            for (i, j, impulse, correction) in responses {
                if let Some(body) = self.bodies.get_mut(i) {
                    if body.body_type == BodyType::Dynamic {
                        body.apply_impulse(-impulse);
                        body.position -= correction / body.mass;
                    }
                }

                if let Some(body) = self.bodies.get_mut(j) {
                    if body.body_type == BodyType::Dynamic {
                        body.apply_impulse(impulse);
                        body.position += correction / body.mass;
                    }
                }
            }
        }

        // Update sleeping state
        for body in self.bodies.iter_mut() {
            if body.body_type == BodyType::Dynamic && !body.sleeping {
                if body.velocity.length_squared() < 0.01
                    && body.angular_velocity.length_squared() < 0.01
                {
                    body.set_sleeping(true);
                }
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
