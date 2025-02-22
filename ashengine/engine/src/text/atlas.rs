use crate::context::Context;
use crate::error::{Result, VulkanError};
use ash::vk;
use std::collections::HashMap;
use std::sync::Arc;

use super::layout::Rect;

#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub advance: f32,
    pub bearing: [f32; 2],
    pub size: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub uv_rect: Rect,
    pub metrics: GlyphMetrics,
}

pub struct FontAtlas {
    texture: vk::Image,
    view: vk::ImageView,
    sampler: vk::Sampler,
    memory: vk::DeviceMemory,
    extent: vk::Extent2D,
    glyph_data: HashMap<char, GlyphInfo>,
    context: Arc<Context>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
}

impl FontAtlas {
    pub fn new(context: Arc<Context>, width: u32, height: u32) -> Result<Self> {
        let device = context.device();
        let extent = vk::Extent2D { width, height };

        // Create texture image
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8_UNORM)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let texture = unsafe {
            device
                .create_image(&image_info, None)
                .map_err(|e| VulkanError::ImageCreation(e.to_string()))?
        };

        // Allocate and bind memory
        let mem_requirements = unsafe { device.get_image_memory_requirements(texture) };
        let memory_properties = unsafe {
            context
                .instance()
                .get_physical_device_memory_properties(context.physical_device())
        };

        // Find suitable memory type index
        let memory_type_index = (0..memory_properties.memory_type_count)
            .find(|i| {
                let suitable = (mem_requirements.memory_type_bits & (1 << i)) != 0;
                let memory_type = memory_properties.memory_types[*i as usize];
                suitable
                    && memory_type
                        .property_flags
                        .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
            })
            .ok_or(VulkanError::NoSuitableMemoryType)?;

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index);

        let memory = unsafe {
            device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| VulkanError::MemoryAllocation(e.to_string()))?
        };

        unsafe {
            device
                .bind_image_memory(texture, memory, 0)
                .map_err(|e| VulkanError::MemoryBinding(e.to_string()))?;
        }

        // Create descriptor set layout
        let binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();

        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&[binding]);

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&layout_info, None)
                .map_err(|e| VulkanError::DescriptorSetLayoutCreation(e.to_string()))?
        };

        // Create descriptor pool
        let pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
        };

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&[pool_size])
            .max_sets(1);

        let descriptor_pool = unsafe {
            device
                .create_descriptor_pool(&pool_info, None)
                .map_err(|e| VulkanError::DescriptorPoolCreation(e.to_string()))?
        };

        // Allocate descriptor set
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&[descriptor_set_layout]);

        let descriptor_set = unsafe {
            device
                .allocate_descriptor_sets(&alloc_info)
                .map_err(|e| VulkanError::DescriptorSetAllocation(e.to_string()))?[0]
        };

        // Create image view
        let view_info = vk::ImageViewCreateInfo::builder()
            .image(texture)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8_UNORM)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let view = unsafe {
            device
                .create_image_view(&view_info, None)
                .map_err(|e| VulkanError::ImageViewCreation(e.to_string()))?
        };

        // Create sampler
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .anisotropy_enable(false)
            .max_anisotropy(1.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0);

        let sampler = unsafe {
            device
                .create_sampler(&sampler_info, None)
                .map_err(|e| VulkanError::SamplerCreation(e.to_string()))?
        };

        // Transition image layout
        let command_pool = unsafe {
            device
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::builder()
                        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
                        .queue_family_index(context.queue_family_index()),
                    None,
                )
                .map_err(|e| VulkanError::CommandPoolCreation(e.to_string()))?
        };

        let command_buffer = unsafe {
            let alloc_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            device
                .allocate_command_buffers(&alloc_info)
                .map_err(|e| VulkanError::CommandBufferAllocation(e.to_string()))?[0]
        };

        unsafe {
            device
                .begin_command_buffer(
                    command_buffer,
                    &vk::CommandBufferBeginInfo::builder()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .map_err(|e| VulkanError::CommandBufferBegin(e.to_string()))?;

            let barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(texture)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .build();

            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );

            device
                .end_command_buffer(command_buffer)
                .map_err(|e| VulkanError::CommandBufferEnd(e.to_string()))?;

            let submit_info = vk::SubmitInfo::builder()
                .command_buffers(&[command_buffer])
                .build();

            device
                .queue_submit(context.graphics_queue(), &[submit_info], vk::Fence::null())
                .map_err(|e| VulkanError::QueueSubmit(e.to_string()))?;

            device
                .queue_wait_idle(context.graphics_queue())
                .map_err(|e| VulkanError::QueueWaitIdle(e.to_string()))?;

            device.destroy_command_pool(command_pool, None);
        }

        // Update descriptor set
        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(view)
            .sampler(sampler)
            .build();

        let write_descriptor_set = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&[image_info])
            .build();

        unsafe {
            device.update_descriptor_sets(&[write_descriptor_set], &[]);
        }

        Ok(Self {
            texture,
            view,
            sampler,
            memory,
            extent,
            glyph_data: HashMap::new(),
            context,
            descriptor_pool,
            descriptor_set_layout,
            descriptor_set,
        })
    }

    pub fn add_glyph(&mut self, c: char, uv_rect: Rect, metrics: GlyphMetrics) {
        self.glyph_data.insert(c, GlyphInfo { uv_rect, metrics });
    }

    pub fn get_glyph(&self, c: char) -> Option<&GlyphInfo> {
        self.glyph_data.get(&c)
    }

    pub fn descriptor_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }

    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }
}

impl Drop for FontAtlas {
    fn drop(&mut self) {
        let device = self.context.device();
        unsafe {
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            device.destroy_sampler(self.sampler, None);
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.texture, None);
            device.free_memory(self.memory, None);
        }
    }
}
