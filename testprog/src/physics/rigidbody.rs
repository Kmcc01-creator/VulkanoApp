use glam::{Quat, Vec3};

#[derive(Debug, Clone, Copy)]
pub enum BodyType {
    Static,    // Immovable objects
    Dynamic,   // Fully simulated objects
    Kinematic, // Moved by user code, affects dynamic bodies
}

impl Default for BodyType {
    fn default() -> Self {
        Self::Dynamic
    }
}

#[derive(Debug, Clone)]
pub struct RigidBody {
    pub body_type: BodyType,
    pub position: Vec3,
    pub rotation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub mass: f32,
    pub inertia: Vec3,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub gravity_scale: f32,
    pub sleeping: bool,
    pub continuous: bool, // Enable continuous collision detection
    pub locked_axes: u8,  // Bitfield for locked translation/rotation axes
}

impl RigidBody {
    pub fn new(body_type: BodyType) -> Self {
        Self {
            body_type,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass: 1.0,
            inertia: Vec3::ONE,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
            sleeping: false,
            continuous: false,
            locked_axes: 0,
        }
    }

    pub fn static_body() -> Self {
        Self::new(BodyType::Static)
    }

    pub fn dynamic_body() -> Self {
        Self::new(BodyType::Dynamic)
    }

    pub fn kinematic_body() -> Self {
        Self::new(BodyType::Kinematic)
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    pub fn with_damping(mut self, linear: f32, angular: f32) -> Self {
        self.linear_damping = linear;
        self.angular_damping = angular;
        self
    }

    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    pub fn with_continuous(mut self, enabled: bool) -> Self {
        self.continuous = enabled;
        self
    }

    // Lock specific axes (bit flags)
    pub const LOCK_TRANSLATION_X: u8 = 1 << 0;
    pub const LOCK_TRANSLATION_Y: u8 = 1 << 1;
    pub const LOCK_TRANSLATION_Z: u8 = 1 << 2;
    pub const LOCK_ROTATION_X: u8 = 1 << 3;
    pub const LOCK_ROTATION_Y: u8 = 1 << 4;
    pub const LOCK_ROTATION_Z: u8 = 1 << 5;

    pub fn lock_axes(mut self, axes: u8) -> Self {
        self.locked_axes = axes;
        self
    }

    // Helper methods for common constraints
    pub fn lock_rotation(mut self) -> Self {
        self.locked_axes |= Self::LOCK_ROTATION_X | Self::LOCK_ROTATION_Y | Self::LOCK_ROTATION_Z;
        self
    }

    pub fn lock_translation(mut self) -> Self {
        self.locked_axes |=
            Self::LOCK_TRANSLATION_X | Self::LOCK_TRANSLATION_Y | Self::LOCK_TRANSLATION_Z;
        self
    }

    pub fn apply_force(&mut self, force: Vec3) {
        if self.body_type == BodyType::Dynamic && !self.sleeping {
            self.linear_velocity += force / self.mass;
        }
    }

    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.body_type == BodyType::Dynamic && !self.sleeping {
            self.linear_velocity += impulse / self.mass;
        }
    }

    pub fn apply_torque(&mut self, torque: Vec3) {
        if self.body_type == BodyType::Dynamic && !self.sleeping {
            self.angular_velocity += torque;
        }
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::dynamic_body()
    }
}
