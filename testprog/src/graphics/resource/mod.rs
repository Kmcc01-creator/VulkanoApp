mod buffer;
mod pool;
mod texture;

pub use buffer::{Buffer, BufferHandle, BufferManager};
pub use pool::{ResourceHandle, ResourcePool};
pub use texture::{Texture, TextureHandle, TextureManager};

use smallvec::SmallVec;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, StandardMemoryAllocator};

use crate::core::error::Error;
use crate::graphics::config::ResourceConfig;

/// Manages all graphics resources including textures, buffers, and memory allocation
pub struct ResourceManager {
    device: Arc<Device>,
    allocator: Arc<StandardMemoryAllocator>,
    texture_manager: TextureManager,
    buffer_manager: BufferManager,
    config: ResourceConfig,
}

impl ResourceManager {
    pub fn new(device: Arc<Device>, config: ResourceConfig) -> Result<Self, Error> {
        let allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let texture_manager = TextureManager::new(
            device.clone(),
            allocator.clone(),
            config.texture_memory_budget,
            config.max_cached_textures,
        )?;

        let buffer_manager = BufferManager::new(device.clone(), allocator.clone())?;

        Ok(Self {
            device,
            allocator,
            texture_manager,
            buffer_manager,
            config,
        })
    }

    /// Get the texture manager
    pub fn textures(&mut self) -> &mut TextureManager {
        &mut self.texture_manager
    }

    /// Get the buffer manager
    pub fn buffers(&mut self) -> &mut BufferManager {
        &mut self.buffer_manager
    }

    /// Cleanup unused resources
    pub fn cleanup(&mut self) {
        self.texture_manager.cleanup();
        self.buffer_manager.cleanup();
    }

    /// Get current memory usage statistics
    pub fn memory_stats(&self) -> ResourceStats {
        ResourceStats {
            texture_memory_used: self.texture_manager.memory_used(),
            buffer_memory_used: self.buffer_manager.memory_used(),
            total_allocations: self.texture_manager.allocation_count()
                + self.buffer_manager.allocation_count(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResourceStats {
    pub texture_memory_used: u64,
    pub buffer_memory_used: u64,
    pub total_allocations: usize,
}

/// Helper for tracking resource lifetimes
#[derive(Debug)]
pub(crate) struct ResourceTracker {
    // Track frame last used
    last_used: u64,
    // Reference count
    ref_count: u32,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            last_used: 0,
            ref_count: 1,
        }
    }

    pub fn increment(&mut self) {
        self.ref_count += 1;
    }

    pub fn decrement(&mut self) -> bool {
        self.ref_count -= 1;
        self.ref_count == 0
    }

    pub fn mark_used(&mut self, frame: u64) {
        self.last_used = frame;
    }

    pub fn is_unused(&self, current_frame: u64, frames_inactive: u64) -> bool {
        self.ref_count == 0 && current_frame.saturating_sub(self.last_used) > frames_inactive
    }
}
