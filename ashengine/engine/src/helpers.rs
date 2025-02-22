use crate::error::{Result, VulkanError};
use ash::vk;

pub fn create_descriptor_pool(
    device: &ash::Device,
    max_sets: u32,
    pool_sizes: &[vk::DescriptorPoolSize],
) -> Result<vk::DescriptorPool> {
    let pool_info = vk::DescriptorPoolCreateInfo::builder()
        .max_sets(max_sets)
        .pool_sizes(pool_sizes);

    unsafe {
        device
            .create_descriptor_pool(&pool_info, None)
            .map_err(|e| VulkanError::DescriptorPoolCreation(e.to_string()))
    }
}

pub fn create_descriptor_set_layout(
    device: &ash::Device,
    bindings: &[vk::DescriptorSetLayoutBinding],
) -> Result<vk::DescriptorSetLayout> {
    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);

    unsafe {
        device
            .create_descriptor_set_layout(&layout_info, None)
            .map_err(|e| VulkanError::DescriptorSetLayoutCreation(e.to_string()))
    }
}

pub fn create_pipeline_layout(
    device: &ash::Device,
    set_layouts: &[vk::DescriptorSetLayout],
) -> Result<vk::PipelineLayout> {
    let layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(set_layouts);

    unsafe {
        device
            .create_pipeline_layout(&layout_info, None)
            .map_err(|e| VulkanError::PipelineLayoutCreation(e.to_string()))
    }
}

pub fn allocate_descriptor_sets(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    layouts: &[vk::DescriptorSetLayout],
) -> Result<Vec<vk::DescriptorSet>> {
    let alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(layouts);

    unsafe {
        device
            .allocate_descriptor_sets(&alloc_info)
            .map_err(|e| VulkanError::DescriptorSetAllocation(e.to_string()))
    }
}

pub fn create_vertex_buffer<T: Copy>(
    device: &ash::Device,
    vertices: &[T],
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_size = (std::mem::size_of::<T>() * vertices.len()) as u64;
    let usage = vk::BufferUsageFlags::VERTEX_BUFFER;
    let memory_properties =
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

    let (buffer, memory) =
        crate::utils::create_buffer(device, buffer_size, usage, memory_properties)?;

    // Copy data to buffer
    unsafe {
        let data_ptr = device
            .map_memory(memory, 0, buffer_size, vk::MemoryMapFlags::empty())
            .map_err(|e| VulkanError::MemoryMapping(e.to_string()))?;

        std::ptr::copy_nonoverlapping(
            vertices.as_ptr() as *const u8,
            data_ptr as *mut u8,
            buffer_size as usize,
        );

        device.unmap_memory(memory);
    }

    Ok((buffer, memory))
}

pub fn create_index_buffer(
    device: &ash::Device,
    indices: &[u32],
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_size = (std::mem::size_of::<u32>() * indices.len()) as u64;
    let usage = vk::BufferUsageFlags::INDEX_BUFFER;
    let memory_properties =
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

    let (buffer, memory) =
        crate::utils::create_buffer(device, buffer_size, usage, memory_properties)?;

    // Copy data to buffer
    unsafe {
        let data_ptr = device
            .map_memory(memory, 0, buffer_size, vk::MemoryMapFlags::empty())
            .map_err(|e| VulkanError::MemoryMapping(e.to_string()))?;

        std::ptr::copy_nonoverlapping(
            indices.as_ptr() as *const u8,
            data_ptr as *mut u8,
            buffer_size as usize,
        );

        device.unmap_memory(memory);
    }

    Ok((buffer, memory))
}

pub fn create_storage_buffer<T: Copy>(
    device: &ash::Device,
    data: &[T],
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_size = (std::mem::size_of::<T>() * data.len()) as u64;
    let usage = vk::BufferUsageFlags::STORAGE_BUFFER;
    let memory_properties =
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

    let (buffer, memory) =
        crate::utils::create_buffer(device, buffer_size, usage, memory_properties)?;

    // Copy data to buffer
    unsafe {
        let data_ptr = device
            .map_memory(memory, 0, buffer_size, vk::MemoryMapFlags::empty())
            .map_err(|e| VulkanError::MemoryMapping(e.to_string()))?;

        std::ptr::copy_nonoverlapping(
            data.as_ptr() as *const u8,
            data_ptr as *mut u8,
            buffer_size as usize,
        );

        device.unmap_memory(memory);
    }

    Ok((buffer, memory))
}
