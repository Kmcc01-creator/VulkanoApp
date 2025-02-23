//! GPU-accelerated particle physics system using Vulkan compute shaders.
//!
//! This module provides a high-performance particle physics implementation with:
//! - Double buffering for efficient GPU-CPU synchronization
//! - Memory pooling and dynamic resizing
//! - Debug visualization and profiling support
//! - Comprehensive error handling and recovery

mod debug;
mod gpu_physics;
mod memory;
mod shaders;

pub use gpu_physics::{GpuPhysicsSystem, Particle, PhysicsError, PushConstants, SystemState};

pub use debug::{DebugStats, DebugVisualization, ParticleDebugView};

pub use memory::{BufferPool, MemoryStats};

/// Configuration options for initializing the physics system
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    /// Initial number of particles
    pub initial_particles: usize,
    /// Enable debug visualization
    pub debug_enabled: bool,
    /// How often to update debug statistics (in frames)
    pub debug_sample_rate: u32,
    /// Initial memory pool size in bytes
    pub initial_pool_size: u64,
    /// Maximum number of recovery attempts before failing
    pub max_recovery_attempts: u32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            initial_particles: 1000,
            debug_enabled: false,
            debug_sample_rate: 60,
            initial_pool_size: 1024 * 1024, // 1MB
            max_recovery_attempts: 3,
        }
    }
}

/// Create a new physics system with the specified configuration
pub fn create_physics_system(
    device: std::sync::Arc<ash::Device>,
    physical_device: ash::vk::PhysicalDevice,
    queue_family_index: u32,
    config: Option<PhysicsConfig>,
) -> Result<(GpuPhysicsSystem, DebugVisualization), PhysicsError> {
    let config = config.unwrap_or_default();

    let mut physics = GpuPhysicsSystem::new(device, physical_device, queue_family_index)?;
    physics.debug_enabled = config.debug_enabled; // Set the debug flag

    let mut debug = DebugVisualization::new(config.debug_sample_rate);
    if config.debug_enabled {
        debug.enable();
    }

    Ok((physics, debug))
}

/// Re-export common types and traits
pub mod prelude {
    pub use super::{
        create_physics_system, DebugStats, DebugVisualization, GpuPhysicsSystem, MemoryStats,
        Particle, PhysicsConfig, PhysicsError,
    };
}

// Private test module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_config() {
        let config = PhysicsConfig::default();
        assert_eq!(config.initial_particles, 1000);
        assert!(!config.debug_enabled);
        assert_eq!(config.debug_sample_rate, 60);
        assert_eq!(config.initial_pool_size, 1024 * 1024);
        assert_eq!(config.max_recovery_attempts, 3);
    }
}
