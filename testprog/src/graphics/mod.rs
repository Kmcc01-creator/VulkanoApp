mod pipeline;
mod renderer;
mod shader;
mod swapchain;
mod vertex;

pub use renderer::RenderContext;
pub use swapchain::SwapchainContext;
pub use vertex::{MeshVertex, SpriteVertex, UiVertex};

use crate::core::error::Error;
use std::sync::Arc;
use vulkano::device::physical::{PhysicalDevice, QueueFamily};
use vulkano::device::DeviceExtensions;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo};
use vulkano::format::Format;
use vulkano::image::Image;
use vulkano::image::ImageUsage;
use vulkano::instance::Surface;
use vulkano::swapchain::CompositeAlpha;
use vulkano::swapchain::{Surface as SwapchainSurface, Swapchain, SwapchainCreateInfo};

pub fn create_logical_device(
    physical_device: Arc<PhysicalDevice>,
    surface: Arc<SwapchainSurface>,
) -> Result<(Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>>), Error> {
    let queue_family = physical_device
        .queue_families()
        .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        .ok_or_else(|| {
            Error::GraphicsInitialization("No suitable queue family found".to_string())
        })?;

    let (device, queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
            enabled_extensions: DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::empty()
            },
            ..Default::default()
        },
    )
    .map_err(|e| Error::GraphicsInitialization(format!("Failed to create device: {}", e)))?;

    Ok((device, queues))
}

pub fn create_swapchain(
    device: Arc<Device>,
    surface: Arc<SwapchainSurface>,
    window_size: [u32; 2],
) -> Result<(Arc<Swapchain>, Vec<Arc<Image>>), Error> {
    let capabilities = surface
        .capabilities(device.physical_device())
        .map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to get surface capabilities: {}", e))
        })?;

    let composite_alpha = capabilities
        .supported_composite_alpha
        .iter()
        .next()
        .unwrap();

    let format = surface
        .formats(device.physical_device())
        .map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to get surface formats: {}", e))
        })?
        .into_iter()
        .next()
        .ok_or_else(|| Error::GraphicsInitialization("No surface formats available".to_string()))?
        .0;

    let (swapchain, images) = Swapchain::new(
        device,
        surface,
        SwapchainCreateInfo {
            min_image_count: capabilities.min_image_count + 1,
            image_format: format,
            image_extent: window_size,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha,
            ..Default::default()
        },
    )
    .map_err(|e| Error::GraphicsInitialization(format!("Failed to create swapchain: {}", e)))?;

    Ok((swapchain, images))
}
