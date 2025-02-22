//! AshEngine - A Vulkan-based graphics engine written in Rust

pub mod commands;
pub mod config;
pub mod context;
pub mod error;
pub mod helpers;
pub mod lighting;
pub mod memory;
pub mod mesh;
pub mod physics;
pub mod pipeline;
pub mod render_pass;
pub mod renderer;
pub mod resource;
pub mod shader;
pub mod swapchain;
pub mod text;

// Re-exports for convenience
pub use error::{Result, VulkanError};
pub use pipeline::Pipeline;
pub use render_pass::RenderPass;
pub use renderer::Renderer;
pub use swapchain::Swapchain;

// Helper functions commonly used across the engine
pub(crate) mod utils {
    use ash::vk;

    pub fn create_buffer(
        device: &ash::Device,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        #[allow(unused_variables)] memory_properties: vk::MemoryPropertyFlags,
    ) -> crate::Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device
                .create_buffer(&buffer_info, None)
                .map_err(|e| crate::VulkanError::BufferCreation(e.to_string()))?
        };

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(0);

        let memory = unsafe {
            device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| crate::VulkanError::MemoryAllocation(e.to_string()))?
        };

        unsafe {
            device
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(|e| crate::VulkanError::MemoryBinding(e.to_string()))?;
        }

        Ok((buffer, memory))
    }

    /// Creates a 2D image with the specified properties.
    /// Currently unused but kept for future texture/image handling features.
    #[allow(dead_code)]
    pub fn create_image(
        device: &ash::Device,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        #[allow(unused_variables)] memory_properties: vk::MemoryPropertyFlags,
    ) -> crate::Result<(vk::Image, vk::DeviceMemory)> {
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let image = unsafe {
            device
                .create_image(&image_info, None)
                .map_err(|e| crate::VulkanError::ImageCreation(e.to_string()))?
        };

        let mem_requirements = unsafe { device.get_image_memory_requirements(image) };

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(0);

        let memory = unsafe {
            device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| crate::VulkanError::MemoryAllocation(e.to_string()))?
        };

        unsafe {
            device
                .bind_image_memory(image, memory, 0)
                .map_err(|e| crate::VulkanError::MemoryBinding(e.to_string()))?;
        }

        Ok((image, memory))
    }

    /// Creates a shader module from raw SPIR-V bytecode.
    /// Currently unused but kept for future shader loading features.
    #[allow(dead_code)]
    pub fn create_shader_module(
        device: &ash::Device,
        code: &[u8],
    ) -> crate::Result<vk::ShaderModule> {
        // Ensure the byte slice length is a multiple of 4
        if code.len() % 4 != 0 {
            return Err(crate::VulkanError::ShaderCreation(
                "Shader code length must be a multiple of 4".to_string(),
            ));
        }

        let code = unsafe {
            std::slice::from_raw_parts(
                code.as_ptr().cast::<u32>(),
                code.len() / std::mem::size_of::<u32>(),
            )
        };

        let create_info = vk::ShaderModuleCreateInfo::builder().code(code);

        unsafe {
            device
                .create_shader_module(&create_info, None)
                .map_err(|e| crate::VulkanError::ShaderCreation(e.to_string()))
        }
    }
}

// Re-export all the types needed for text rendering
pub use text::{
    ndc_to_pixel, pixel_to_ndc, FontAtlas, TextAlignment, TextConfig, TextElement, TextLayout,
    TextPicker,
};
