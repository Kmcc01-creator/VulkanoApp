use glam::{Vec2, Vec3};

#[derive(Debug, Clone)]
pub enum ColliderType {
    Box(Vec3),   // Half-extents
    Sphere(f32), // Radius
    Capsule { radius: f32, height: f32 },
    Box2D(Vec2), // Half-extents for 2D
    Circle(f32), // Radius for 2D
}

#[derive(Debug, Clone)]
pub struct ColliderMaterial {
    pub restitution: f32, // Bounciness (0-1)
    pub friction: f32,    // Friction coefficient
    pub density: f32,     // Mass per unit volume
}

impl Default for ColliderMaterial {
    fn default() -> Self {
        Self {
            restitution: 0.5,
            friction: 0.5,
            density: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Collider {
    pub shape: ColliderType,
    pub material: ColliderMaterial,
    pub is_sensor: bool,     // If true, detects collisions but doesn't respond
    pub collision_mask: u32, // Collision layer mask
}

impl Collider {
    pub fn new(shape: ColliderType) -> Self {
        Self {
            shape,
            material: ColliderMaterial::default(),
            is_sensor: false,
            collision_mask: 0xFFFFFFFF, // Collide with everything by default
        }
    }

    pub fn with_material(mut self, material: ColliderMaterial) -> Self {
        self.material = material;
        self
    }

    pub fn with_sensor(mut self, is_sensor: bool) -> Self {
        self.is_sensor = is_sensor;
        self
    }

    pub fn with_collision_mask(mut self, mask: u32) -> Self {
        self.collision_mask = mask;
        self
    }

    // Helper methods for common shapes
    pub fn box_shape(half_extents: Vec3) -> Self {
        Self::new(ColliderType::Box(half_extents))
    }

    pub fn sphere(radius: f32) -> Self {
        Self::new(ColliderType::Sphere(radius))
    }

    pub fn capsule(radius: f32, height: f32) -> Self {
        Self::new(ColliderType::Capsule { radius, height })
    }

    pub fn box_2d(half_extents: Vec2) -> Self {
        Self::new(ColliderType::Box2D(half_extents))
    }

    pub fn circle(radius: f32) -> Self {
        Self::new(ColliderType::Circle(radius))
    }

    // Utility methods for collision detection
    pub fn compute_aabb(&self, position: Vec3, rotation: glam::Quat) -> AABB {
        match self.shape {
            ColliderType::Box(half_extents) => {
                let transformed_extents = rotation * half_extents;
                let abs_extents = Vec3::new(
                    transformed_extents.x.abs(),
                    transformed_extents.y.abs(),
                    transformed_extents.z.abs(),
                );
                AABB {
                    min: position - abs_extents,
                    max: position + abs_extents,
                }
            }
            ColliderType::Sphere(radius) => AABB {
                min: position - Vec3::splat(radius),
                max: position + Vec3::splat(radius),
            },
            ColliderType::Capsule { radius, height } => {
                let half_height = height * 0.5;
                let extent = Vec3::new(radius, half_height + radius, radius);
                AABB {
                    min: position - extent,
                    max: position + extent,
                }
            }
            ColliderType::Box2D(half_extents) => {
                let half_extents = Vec3::new(half_extents.x, half_extents.y, 0.0);
                let transformed_extents = rotation * half_extents;
                let abs_extents = Vec3::new(
                    transformed_extents.x.abs(),
                    transformed_extents.y.abs(),
                    0.0,
                );
                AABB {
                    min: position - abs_extents,
                    max: position + abs_extents,
                }
            }
            ColliderType::Circle(radius) => AABB {
                min: position - Vec3::new(radius, radius, 0.0),
                max: position + Vec3::new(radius, radius, 0.0),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }
}
