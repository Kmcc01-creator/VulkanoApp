use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, StandardMemoryAllocator};

use crate::core::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHandle<T>(u64, std::marker::PhantomData<T>);

pub struct ResourcePool<T> {
    resources: HashMap<ResourceHandle<T>, T>,
    free_list: Vec<ResourceHandle<T>>,
    next_handle: u64,
}

#[derive(Debug)]
pub struct PoolStats {
    pub active_resources: usize,
    pub free_resources: usize,
    pub total_resources: usize,
}

impl<T> ResourcePool<T> {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            free_list: Vec::new(),
            next_handle: 1,
        }
    }

    pub fn allocate(&mut self, resource: T) -> ResourceHandle<T> {
        if let Some(handle) = self.free_list.pop() {
            self.resources.insert(handle, resource);
            handle
        } else {
            let handle = ResourceHandle(self.next_handle, std::marker::PhantomData);
            self.next_handle += 1;
            self.resources.insert(handle, resource);
            handle
        }
    }

    pub fn deallocate(&mut self, handle: ResourceHandle<T>) -> Option<T> {
        if let Some(resource) = self.resources.remove(&handle) {
            self.free_list.push(handle);
            Some(resource)
        } else {
            None
        }
    }

    pub fn get(&self, handle: ResourceHandle<T>) -> Option<&T> {
        self.resources.get(&handle)
    }

    pub fn get_mut(&mut self, handle: ResourceHandle<T>) -> Option<&mut T> {
        self.resources.get_mut(&handle)
    }

    pub fn contains(&self, handle: ResourceHandle<T>) -> bool {
        self.resources.contains_key(&handle)
    }

    pub fn clear(&mut self) {
        self.resources.clear();
        self.free_list.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ResourceHandle<T>, &T)> {
        self.resources.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ResourceHandle<T>, &mut T)> {
        self.resources.iter_mut()
    }

    pub fn stats(&self) -> PoolStats {
        PoolStats {
            active_resources: self.resources.len(),
            free_resources: self.free_list.len(),
            total_resources: self.resources.len() + self.free_list.len(),
        }
    }
}

impl<T> Default for ResourcePool<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Helper trait for pooled resources
pub trait PooledResource: Sized {
    type CreateInfo;
    type Error;

    fn create(device: Arc<Device>, info: &Self::CreateInfo) -> Result<Self, Self::Error>;
    fn destroy(&mut self);
}

// Generic resource pool with Vulkan device dependency
pub struct VulkanResourcePool<T: PooledResource> {
    device: Arc<Device>,
    pool: ResourcePool<T>,
}

impl<T: PooledResource> VulkanResourcePool<T> {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            pool: ResourcePool::new(),
        }
    }

    pub fn allocate(&mut self, info: &T::CreateInfo) -> Result<ResourceHandle<T>, T::Error> {
        let resource = T::create(self.device.clone(), info)?;
        Ok(self.pool.allocate(resource))
    }

    pub fn deallocate(&mut self, handle: ResourceHandle<T>) {
        if let Some(mut resource) = self.pool.deallocate(handle) {
            resource.destroy();
        }
    }

    pub fn get(&self, handle: ResourceHandle<T>) -> Option<&T> {
        self.pool.get(handle)
    }

    pub fn get_mut(&mut self, handle: ResourceHandle<T>) -> Option<&mut T> {
        self.pool.get_mut(handle)
    }

    pub fn clear(&mut self) {
        for (_, mut resource) in self.pool.resources.drain() {
            resource.destroy();
        }
        self.pool.clear();
    }

    pub fn stats(&self) -> PoolStats {
        self.pool.stats()
    }
}

// Implementation for common cleanup patterns
impl<T: PooledResource> Drop for VulkanResourcePool<T> {
    fn drop(&mut self) {
        self.clear();
    }
}
