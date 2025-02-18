use std::collections::HashMap;
use std::sync::Arc;
use vulkano::buffer::{Buffer as VkBuffer, BufferCreateInfo, BufferUsage};
use vulkano::device::Device;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, StandardMemoryAllocator};

use super::ResourceTracker;
use crate::core::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle(u64);

pub struct Buffer {
    buffer: Arc<VkBuffer>,
    size: u64,
    usage: BufferUsage,
}

pub struct BufferManager {
    device: Arc<Device>,
    allocator: Arc<StandardMemoryAllocator>,
    buffers: HashMap<BufferHandle, (Buffer, ResourceTracker)>,
    memory_used: u64,
    next_handle: u64,
    current_frame: u64,
}

#[derive(Debug, Clone)]
pub struct BufferCreateInfo {
    pub size: u64,
    pub usage: BufferUsage,
}

// Pool-related structures for buffer reuse
#[derive(Debug)]
struct BufferPool {
    available: Vec<BufferHandle>,
    size: u64,
    usage: BufferUsage,
}

impl BufferManager {
    pub fn new(
        device: Arc<Device>,
        allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self, Error> {
        Ok(Self {
            device,
            allocator,
            buffers: HashMap::new(),
            memory_used: 0,
            next_handle: 1,
            current_frame: 0,
        })
    }

    pub fn create_buffer(&mut self, info: BufferCreateInfo) -> Result<BufferHandle, Error> {
        let buffer = VkBuffer::new(
            self.allocator.clone(),
            BufferCreateInfo {
                usage: info.usage,
                size: info.size,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .map_err(|e| Error::ResourceCreation(format!("Failed to create buffer: {}", e)))?;

        let buffer_obj = Buffer {
            buffer,
            size: info.size,
            usage: info.usage,
        };

        let handle = BufferHandle(self.next_handle);
        self.next_handle += 1;

        self.memory_used += info.size;
        self.buffers
            .insert(handle, (buffer_obj, ResourceTracker::new()));

        Ok(handle)
    }

    pub fn get_buffer(&self, handle: BufferHandle) -> Option<&Buffer> {
        self.buffers.get(&handle).map(|(buffer, _)| buffer)
    }

    pub fn get_buffer_mut(&mut self, handle: BufferHandle) -> Option<&mut Buffer> {
        self.buffers.get_mut(&handle).map(|(buffer, _)| buffer)
    }

    pub fn destroy_buffer(&mut self, handle: BufferHandle) {
        if let Some((buffer, _)) = self.buffers.remove(&handle) {
            self.memory_used -= buffer.size;
        }
    }

    pub fn cleanup(&mut self) {
        let mut to_remove = Vec::new();

        for (handle, (buffer, tracker)) in &self.buffers {
            if tracker.is_unused(self.current_frame, 60) {
                // Remove after ~1 second unused
                to_remove.push(*handle);
                self.memory_used -= buffer.size;
            }
        }

        for handle in to_remove {
            self.buffers.remove(&handle);
        }
    }

    pub fn mark_frame_complete(&mut self) {
        self.current_frame += 1;
    }

    pub fn memory_used(&self) -> u64 {
        self.memory_used
    }

    pub fn allocation_count(&self) -> usize {
        self.buffers.len()
    }
}

impl Buffer {
    pub fn buffer(&self) -> &Arc<VkBuffer> {
        &self.buffer
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn usage(&self) -> BufferUsage {
        self.usage
    }
}

// Buffer pool implementation for efficient reuse
impl BufferPool {
    pub fn new(size: u64, usage: BufferUsage) -> Self {
        Self {
            available: Vec::new(),
            size,
            usage,
        }
    }

    pub fn add_buffer(&mut self, handle: BufferHandle) {
        self.available.push(handle);
    }

    pub fn get_buffer(&mut self) -> Option<BufferHandle> {
        self.available.pop()
    }

    pub fn matches(&self, size: u64, usage: BufferUsage) -> bool {
        self.size >= size && self.usage.contains(usage)
    }
}

// Helper functions for buffer operations
pub fn calculate_aligned_size(size: u64, alignment: u64) -> u64 {
    (size + alignment - 1) & !(alignment - 1)
}

pub fn is_size_valid(size: u64) -> bool {
    size > 0 && size <= u64::MAX
}
