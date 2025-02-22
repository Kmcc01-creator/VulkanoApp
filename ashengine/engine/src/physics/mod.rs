pub mod collision;
pub mod constraints;
pub mod physics;
pub mod solver;
pub mod spatial;

// Re-export commonly used types
pub use collision::{BoundingVolume, CollisionManifold};
pub use constraints::Constraint;
pub use physics::{PhysicsObject, PhysicsWorld};
pub use solver::{ConstraintSolver, IslandSolver};
pub use spatial::{BroadPhase, SpatialGrid};
