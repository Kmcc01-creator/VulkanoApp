use std::collections::HashMap;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::{Image, ImageCreateInfo, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, StandardMemoryAllocator};

use super::ResourceTracker;
use crate::core::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(u64);

pub struct Texture {
    image: Arc<Image>,
    format: Format,
    width: u32,
    height: u32,
    mip_levels: u32,
    memory_size: u64,
}

pub struct TextureManager {
    device: Arc<Device>,
    allocator: Arc<StandardMemoryAllocator>,
    textures: HashMap<TextureHandle, (Texture, ResourceTracker)>,
    memory_budget: u64,
    memory_used: u64,
    max_textures: u32,
    next_handle: u64,
    current_frame: u64,
}

#[derive(Debug, Clone)]
pub struct TextureCreateInfo {
    pub width: u32,
    pub height: u32,
    pub format: Format,
    pub mip_levels: u32,
    pub usage: ImageUsage,
}

impl TextureManager {
    pub fn new(
        device: Arc<Device>,
        allocator: Arc<StandardMemoryAllocator>,
        memory_budget: u32,
        max_textures: u32,
    ) -> Result<Self, Error> {
        Ok(Self {
            device,
            allocator,
            textures: HashMap::new(),
            memory_budget: memory_budget as u64 * 1024 * 1024, // Convert MB to bytes
            memory_used: 0,
            max_textures,
            next_handle: 1,
            current_frame: 0,
        })
    }

    pub fn create_texture(&mut self, info: TextureCreateInfo) -> Result<TextureHandle, Error> {
        // Check memory budget
        let estimated_size = estimate_texture_size(&info);
        if self.memory_used + estimated_size > self.memory_budget {
            // Try to free some memory
            self.cleanup();

            if self.memory_used + estimated_size > self.memory_budget {
                return Err(Error::OutOfMemory("Texture memory budget exceeded".into()));
            }
        }

        // Check texture count limit
        if self.textures.len() >= self.max_textures as usize {
            return Err(Error::OutOfMemory("Maximum texture count exceeded".into()));
        }

        let image = Image::new(
            self.allocator.clone(),
            ImageCreateInfo {
                format: info.format,
                extent: [info.width, info.height, 1],
                usage: info.usage,
                mip_levels: info.mip_levels,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .map_err(|e| Error::ResourceCreation(format!("Failed to create texture: {}", e)))?;

        let texture = Texture {
            image,
            format: info.format,
            width: info.width,
            height: info.height,
            mip_levels: info.mip_levels,
            memory_size: estimated_size,
        };

        let handle = TextureHandle(self.next_handle);
        self.next_handle += 1;

        self.memory_used += estimated_size;
        self.textures
            .insert(handle, (texture, ResourceTracker::new()));

        Ok(handle)
    }

    pub fn get_texture(&self, handle: TextureHandle) -> Option<&Texture> {
        self.textures.get(&handle).map(|(texture, _)| texture)
    }

    pub fn get_texture_mut(&mut self, handle: TextureHandle) -> Option<&mut Texture> {
        self.textures.get_mut(&handle).map(|(texture, _)| texture)
    }

    pub fn destroy_texture(&mut self, handle: TextureHandle) {
        if let Some((texture, _)) = self.textures.remove(&handle) {
            self.memory_used -= texture.memory_size;
        }
    }

    pub fn cleanup(&mut self) {
        let mut to_remove = Vec::new();

        for (handle, (texture, tracker)) in &self.textures {
            if tracker.is_unused(self.current_frame, 60) {
                // Remove after ~1 second unused
                to_remove.push(*handle);
                self.memory_used -= texture.memory_size;
            }
        }

        for handle in to_remove {
            self.textures.remove(&handle);
        }
    }

    pub fn mark_frame_complete(&mut self) {
        self.current_frame += 1;
    }

    pub fn memory_used(&self) -> u64 {
        self.memory_used
    }

    pub fn allocation_count(&self) -> usize {
        self.textures.len()
    }
}

impl Texture {
    pub fn image(&self) -> &Arc<Image> {
        &self.image
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn dimensions(&self) -> [u32; 2] {
        [self.width, self.height]
    }

    pub fn mip_levels(&self) -> u32 {
        self.mip_levels
    }
}

fn estimate_texture_size(info: &TextureCreateInfo) -> u64 {
    let pixel_size = match info.format {
        Format::R8_UNORM => 1,
        Format::R8G8_UNORM => 2,
        Format::R8G8B8_UNORM => 3,
        Format::R8G8B8A8_UNORM => 4,
        Format::R16_SFLOAT => 2,
        Format::R16G16_SFLOAT => 4,
        Format::R16G16B16_SFLOAT => 6,
        Format::R16G16B16A16_SFLOAT => 8,
        Format::R32_SFLOAT => 4,
        Format::R32G32_SFLOAT => 8,
        Format::R32G32B32_SFLOAT => 12,
        Format::R32G32B32A32_SFLOAT => 16,
        _ => 4, // Default to 4 bytes per pixel
    };

    let mut total_size = 0;
    let mut width = info.width;
    let mut height = info.height;

    for _ in 0..info.mip_levels {
        total_size += width * height * pixel_size;
        if width > 1 {
            width /= 2;
        }
        if height > 1 {
            height /= 2;
        }
    }

    total_size as u64
}
