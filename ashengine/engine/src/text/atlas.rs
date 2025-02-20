use crate::error::{Result, VulkanError};
use crate::text::{GlyphMetrics, Rect};
use ash::{vk, Device};
use std::{collections::HashMap, sync::Arc};

pub struct FontAtlas {
    texture: vk::Image,
    view: vk::ImageView,
    sampler: vk::Sampler,
    memory: vk::DeviceMemory,
    extent: vk::Extent2D,
    glyph_data: HashMap<char, GlyphInfo>,
    device: Arc<Device>,
}

#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub uv_rect: Rect,
    pub metrics: GlyphMetrics,
}

impl FontAtlas {
    pub fn new(device: Arc<Device>, width: u32, height: u32) -> Result<Self> {
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
        let memory_type = vk::MemoryPropertyFlags::DEVICE_LOCAL;

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(0); // You'll need to find proper memory type index

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

        Ok(Self {
            texture,
            view,
            sampler,
            memory,
            extent,
            glyph_data: HashMap::new(),
            device,
        })
    }

    pub fn add_glyph(&mut self, c: char, info: GlyphInfo) {
        self.glyph_data.insert(c, info);
    }

    pub fn get_glyph(&self, c: char) -> Option<&GlyphInfo> {
        self.glyph_data.get(&c)
    }

    pub fn image(&self) -> vk::Image {
        self.texture
    }

    pub fn view(&self) -> vk::ImageView {
        self.view
    }

    pub fn sampler(&self) -> vk::Sampler {
        self.sampler
    }
}

impl Drop for FontAtlas {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_image_view(self.view, None);
            self.device.destroy_image(self.texture, None);
            self.device.free_memory(self.memory, None);
        }
    }
}
