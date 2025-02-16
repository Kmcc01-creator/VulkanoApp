use glam::{Quat, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BodyType {
    Static,
    Dynamic,
    Kinematic,
}

#[derive(Debug)]
pub struct RigidBody {
    pub body_type: BodyType,
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub force: Vec3,
    pub torque: Vec3,
    pub mass: f32,
    pub inertia: f32,
    pub restitution: f32,
    pub friction: f32,
    pub sleeping: bool,
    pub gravity_scale: f32,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            body_type: BodyType::Static,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
            mass: 1.0,
            inertia: 1.0,
            restitution: 0.5,
            friction: 0.5,
            sleeping: false,
            gravity_scale: 1.0,
        }
    }
}

impl RigidBody {
    pub fn new(body_type: BodyType) -> Self {
        Self {
            body_type,
            ..Default::default()
        }
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }

    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.max(0.0);
        self
    }

    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    pub fn apply_force(&mut self, force: Vec3) {
        if self.body_type == BodyType::Dynamic && !self.sleeping {
            self.force += force;
        }
    }

    pub fn apply_torque(&mut self, torque: Vec3) {
        if self.body_type == BodyType::Dynamic && !self.sleeping {
            self.torque += torque;
        }
    }

    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.body_type == BodyType::Dynamic && !self.sleeping {
            self.velocity += impulse / self.mass;
        }
    }

    pub fn apply_angular_impulse(&mut self, impulse: Vec3) {
        if self.body_type == BodyType::Dynamic && !self.sleeping {
            self.angular_velocity += impulse / self.inertia;
        }
    }

    pub fn set_sleeping(&mut self, sleeping: bool) {
        self.sleeping = sleeping;
        if !sleeping {
            self.velocity = Vec3::ZERO;
            self.angular_velocity = Vec3::ZERO;
            self.force = Vec3::ZERO;
            self.torque = Vec3::ZERO;
        }
    }
}
