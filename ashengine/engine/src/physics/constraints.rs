use crate::physics::physics::PhysicsObject;
use glam::{Vec3, Vec4};
use std::cell::RefCell;

pub trait Constraint {
    fn project(&self, objects: &mut Vec<RefCell<PhysicsObject>>);
}

pub struct DistanceConstraint {
    object1_index: usize,
    object2_index: usize,
    rest_distance: f32,
}

impl DistanceConstraint {
    pub fn new(object1_index: usize, object2_index: usize, rest_distance: f32) -> Self {
        DistanceConstraint {
            object1_index,
            object2_index,
            rest_distance,
        }
    }
}

impl Constraint for DistanceConstraint {
    fn project(&self, objects: &mut Vec<RefCell<PhysicsObject>>) {
        let obj1 = objects[self.object1_index].borrow();
        let obj2 = objects[self.object2_index].borrow();

        match (&*obj1, &*obj2) {
            (
                PhysicsObject::RigidBody { position: p1, .. },
                PhysicsObject::RigidBody { position: p2, .. },
            ) => {
                // ... (Logic for two rigid bodies, similar to before) ...
                let delta = *p2 - *p1;
                let distance = delta.length();
                if distance == 0.0 {
                    // Avoid division by zero
                    return;
                }
                let correction = delta * (1.0 - self.rest_distance / distance);
                let total_mass = match &*objects[self.object1_index].borrow_mut() {
                    PhysicsObject::RigidBody { mass, .. } => *mass,
                    _ => 0.0, //should not happen
                } + match &*objects[self.object2_index].borrow_mut() {
                    PhysicsObject::RigidBody { mass, .. } => *mass,
                    _ => 0.0, //should not happen
                };

                if total_mass == 0.0 {
                    return;
                }
                let mut obj1_mut = objects[self.object1_index].borrow_mut();
                let mut obj2_mut = objects[self.object2_index].borrow_mut();

                if let PhysicsObject::RigidBody { position, .. } = &mut *obj1_mut {
                    *position += correction
                        * (match &*obj2_mut {
                            PhysicsObject::RigidBody { mass, .. } => *mass,
                            _ => 0.0,
                        } / total_mass);
                }
                if let PhysicsObject::RigidBody { position, .. } = &mut *obj2_mut {
                    *position -= correction
                        * (match &*obj1_mut {
                            PhysicsObject::RigidBody { mass, .. } => *mass,
                            _ => 0.0,
                        } / total_mass);
                }
            }
            (
                PhysicsObject::DeformableBody {
                    positions: p1s,
                    masses: m1s,
                    ..
                },
                PhysicsObject::DeformableBody {
                    positions: p2s,
                    masses: m2s,
                    ..
                },
            ) => {
                // ... (Logic for two deformable bodies - more complex, likely involves iterating over particles) ...
                // This is a placeholder, needs proper implementation based on how particles are indexed
                // You'd likely need to know *which* particles are connected by this constraint.
                // This is a good example of where having particle indices within the DeformableBody
                // variant would be helpful.
            }
            (PhysicsObject::RigidBody { .. }, PhysicsObject::DeformableBody { .. })
            | (PhysicsObject::DeformableBody { .. }, PhysicsObject::RigidBody { .. }) => {
                // ... (Logic for rigid-deformable interaction) ...
                // This is also a placeholder.  The implementation would depend on how you want
                // rigid bodies and deformable bodies to interact.
            }
        }
    }
}

// --- Collision Constraint (Simplified AABB) ---
pub struct CollisionConstraint {
    object1_index: usize,
    object2_index: usize,
}

impl CollisionConstraint {
    pub fn new(object1_index: usize, object2_index: usize) -> Self {
        CollisionConstraint {
            object1_index,
            object2_index,
        }
    }
}

impl Constraint for CollisionConstraint {
    fn project(&self, objects: &mut Vec<RefCell<PhysicsObject>>) {
        let mut obj1 = objects[self.object1_index].borrow_mut();
        let mut obj2 = objects[self.object2_index].borrow_mut();

        // AABB collision check (simplified from existing code)
        let (overlap_x, overlap_y, overlap_z, axis) = match (&mut *obj1, &mut *obj2) {
            (
                PhysicsObject::RigidBody {
                    position: p1,
                    bounding_box: bb1,
                    ..
                },
                PhysicsObject::RigidBody {
                    position: p2,
                    bounding_box: bb2,
                    ..
                },
            ) => {
                let overlap_x = (p1.x - bb1.w).max(p2.x - bb2.w) - (p1.x + bb1.w).min(p2.x + bb2.w);
                let overlap_y = (p1.y - bb1.w).max(p2.y - bb2.w) - (p1.y + bb1.w).min(p2.y + bb2.w);
                let overlap_z = (p1.z - bb1.w).max(p2.z - bb2.w) - (p1.z + bb1.w).min(p2.z + bb2.w);

                if overlap_x < 0.0 || overlap_y < 0.0 || overlap_z < 0.0 {
                    (0.0, 0.0, 0.0, Vec3::ZERO) // No collision, return early
                } else {
                    let mut min_overlap = overlap_x;
                    let mut axis = Vec3::X;

                    if overlap_y < min_overlap {
                        min_overlap = overlap_y;
                        axis = Vec3::Y;
                    }
                    if overlap_z < min_overlap {
                        min_overlap = overlap_z;
                        axis = Vec3::Z;
                    }
                    (overlap_x, overlap_y, overlap_z, axis)
                }
            }
            _ => (0.0, 0.0, 0.0, Vec3::ZERO), // Handle other cases (e.g., deformable bodies) later
        };

        if axis == Vec3::ZERO {
            return; // No collision
        }

        // Move objects apart along the collision normal
        let separation = axis * (overlap_x.min(overlap_y).min(overlap_z));

        // Need to handle mass correctly for different object types.
        match (&mut *obj1, &mut *obj2) {
            (
                PhysicsObject::RigidBody {
                    position: p1,
                    mass: m1,
                    ..
                },
                PhysicsObject::RigidBody {
                    position: p2,
                    mass: m2,
                    ..
                },
            ) => {
                let total_mass = *m1 + *m2;
                if total_mass == 0.0 {
                    return;
                }
                if p1.dot(axis) > p2.dot(axis) {
                    *p1 += separation * (*m2 / total_mass);
                    *p2 -= separation * (*m1 / total_mass);
                } else {
                    *p1 -= separation * (*m2 / total_mass);
                    *p2 += separation * (*m1 / total_mass);
                }
            }
            _ => (), // Handle other cases later
        }
    }
}
