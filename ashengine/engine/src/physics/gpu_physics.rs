use crate::physics::memory::{BufferPool, MemoryStats};
use crate::physics::shaders::ShaderModule;
use ash::{self, vk};
use std::ptr;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub use crate::physics::debug::{DebugStats, DebugVisualization};

#[derive(Debug)]
pub enum PhysicsError {
    DeviceLost(String),
    OutOfMemory(String),
    InitializationFailed(String),
    InvalidOperation(String),
    BufferOverflow(String),
    SynchronizationError(String),
}

impl std::error::Error for PhysicsError {}

impl std::fmt::Display for PhysicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhysicsError::DeviceLost(msg) => write!(f, "Device Lost: {}", msg),
            PhysicsError::OutOfMemory(msg) => write!(f, "Out of Memory: {}", msg),
            PhysicsError::InitializationFailed(msg) => write!(f, "Initialization Failed: {}", msg),
            PhysicsError::InvalidOperation(msg) => write!(f, "Invalid Operation: {}", msg),
            PhysicsError::BufferOverflow(msg) => write!(f, "Buffer Overflow: {}", msg),
            PhysicsError::SynchronizationError(msg) => write!(f, "Synchronization Error: {}", msg),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub position: [f32; 4], // Using vec4 for GPU alignment
    pub velocity: [f32; 4], // Using vec4 for GPU alignment
}

struct ParticleBufferPair {
    front: (vk::Buffer, vk::DeviceMemory, vk::DeviceSize),
    back: (vk::Buffer, vk::DeviceMemory, vk::DeviceSize),
    mapped_front: *mut std::ffi::c_void,
    mapped_back: *mut std::ffi::c_void,
}

pub struct ParticleDescriptorSets {
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    sets: Vec<vk::DescriptorSet>,
}

pub struct SynchronizationPrimitives {
    compute_fence: vk::Fence,
    compute_semaphore: vk::Semaphore,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
}

#[derive(Debug)]
pub struct SystemState {
    pub is_initialized: bool,
    pub last_error: Option<PhysicsError>,
    pub recovery_attempts: u32,
    pub needs_reset: bool,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            is_initialized: false,
            last_error: None,
            recovery_attempts: 0,
            needs_reset: false,
        }
    }
}

pub struct GpuPhysicsSystem {
    device: Arc<ash::Device>,
    physical_device: vk::PhysicalDevice,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    particle_buffers: Option<ParticleBufferPair>,
    buffer_pool: BufferPool,
    buffer_size: vk::DeviceSize,
    descriptor_sets: Option<ParticleDescriptorSets>,
    sync_primitives: Option<SynchronizationPrimitives>,
    compute_pipeline: Option<vk::Pipeline>,
    pipeline_layout: Option<vk::PipelineLayout>,
    compute_queue: vk::Queue,
    queue_family_index: u32,
    current_frame: usize,
    state: SystemState,
    max_recovery_attempts: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PushConstants {
    delta_time: f32,
    max_velocity: f32,
    bounds: [f32; 2],
}

impl GpuPhysicsSystem {
    pub fn new(
        device: Arc<ash::Device>,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<Self, PhysicsError> {
        unsafe {
            let memory_properties = device.get_physical_device_memory_properties(physical_device);
            let compute_queue = device.get_device_queue(queue_family_index, 0);

            // Create buffer pool with initial size
            let initial_pool_size = 1024 * 1024; // 1MB initial size
            let buffer_pool =
                BufferPool::new(device.clone(), queue_family_index, initial_pool_size)?;

            Ok(Self {
                device,
                physical_device,
                memory_properties,
                particle_buffers: None,
                buffer_pool,
                buffer_size: 0,
                descriptor_sets: None,
                sync_primitives: None,
                compute_pipeline: None,
                pipeline_layout: None,
                compute_queue,
                queue_family_index,
                current_frame: 0,
                state: SystemState::default(),
                max_recovery_attempts: 3,
            })
        }
    }

    pub fn initialize(
        &mut self,
        particle_count: usize,
        shader_module: ShaderModule,
    ) -> Result<(), PhysicsError> {
        if self.state.needs_reset {
            self.try_recover()?;
        }

        let buffer_size = (particle_count * std::mem::size_of::<Particle>()) as u64;

        // Create particle buffers using buffer pool
        let (front_buffer, front_memory, front_offset) = self.buffer_pool.allocate_buffer(
            buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        )?;

        let (back_buffer, back_memory, back_offset) = self.buffer_pool.allocate_buffer(
            buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        )?;

        // Map buffers
        unsafe {
            let front_ptr = self
                .device
                .map_memory(
                    front_memory,
                    front_offset,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| {
                    PhysicsError::InitializationFailed(format!(
                        "Failed to map front buffer memory: {}",
                        e
                    ))
                })?;

            let back_ptr = self
                .device
                .map_memory(
                    back_memory,
                    back_offset,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| {
                    PhysicsError::InitializationFailed(format!(
                        "Failed to map back buffer memory: {}",
                        e
                    ))
                })?;

            self.particle_buffers = Some(ParticleBufferPair {
                front: (front_buffer, front_memory, front_offset),
                back: (back_buffer, back_memory, back_offset),
                mapped_front: front_ptr,
                mapped_back: back_ptr,
            });
        }

        self.buffer_size = buffer_size;

        // Create rest of resources
        self.create_descriptor_sets()?;
        self.create_compute_pipeline(shader_module)?;
        self.create_sync_primitives()?;

        self.state.is_initialized = true;
        Ok(())
    }

    pub fn resize(&mut self, new_particle_count: usize) -> Result<(), PhysicsError> {
        let new_size = (new_particle_count * std::mem::size_of::<Particle>()) as u64;

        // Free old buffers
        if let Some(buffers) = &self.particle_buffers {
            self.buffer_pool
                .free_buffer(buffers.front.0, buffers.front.1, buffers.front.2);
            self.buffer_pool
                .free_buffer(buffers.back.0, buffers.back.1, buffers.back.2);
        }

        // Allocate new buffers
        let (front_buffer, front_memory, front_offset) = self.buffer_pool.allocate_buffer(
            new_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        )?;

        let (back_buffer, back_memory, back_offset) = self.buffer_pool.allocate_buffer(
            new_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        )?;

        // Map new buffers
        unsafe {
            let front_ptr = self
                .device
                .map_memory(
                    front_memory,
                    front_offset,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| {
                    PhysicsError::InitializationFailed(format!(
                        "Failed to map front buffer memory: {}",
                        e
                    ))
                })?;

            let back_ptr = self
                .device
                .map_memory(
                    back_memory,
                    back_offset,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| {
                    PhysicsError::InitializationFailed(format!(
                        "Failed to map back buffer memory: {}",
                        e
                    ))
                })?;

            self.particle_buffers = Some(ParticleBufferPair {
                front: (front_buffer, front_memory, front_offset),
                back: (back_buffer, back_memory, back_offset),
                mapped_front: front_ptr,
                mapped_back: back_ptr,
            });
        }

        self.buffer_size = new_size;

        // Update descriptor sets
        self.update_descriptor_sets()?;

        Ok(())
    }

    pub fn get_memory_stats(&self) -> MemoryStats {
        self.buffer_pool.get_memory_stats()
    }

    // ... [Previous implementations with updated error handling] ...

    pub fn cleanup(&mut self) {
        if let Some(buffers) = &self.particle_buffers {
            unsafe {
                // Unmap memory
                self.device.unmap_memory(buffers.front.1);
                self.device.unmap_memory(buffers.back.1);
            }
        }

        // Cleanup buffer pool
        self.buffer_pool.cleanup();

        // ... [Rest of cleanup] ...
    }
}

impl Drop for GpuPhysicsSystem {
    fn drop(&mut self) {
        self.cleanup();
    }
}
