pub mod config;
mod performance;
mod pipeline;
mod renderer;
mod shader;
mod swapchain;
mod vertex;

// Re-export commonly used types
pub use config::{GraphicsConfig, GraphicsConfigBuilder};
pub use performance::{AsyncLoader, BatchRenderer, PerformanceManager};
pub use pipeline::{PipelineBuilder, PipelineManager};
pub use renderer::RenderContext;
pub use swapchain::SwapchainContext;
pub use vertex::prelude::*;

use crate::core::error::Error;
use std::sync::Arc;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::DeviceExtensions;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::image::Image;
use vulkano::image::ImageUsage;
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};

/// Create logical device and queues
pub fn create_logical_device(
    physical_device: Arc<PhysicalDevice>,
    surface: Arc<Surface>,
) -> Result<(Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>>), Error> {
    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(i, queue_family)| {
            queue_family.queue_flags.intersects(QueueFlags::GRAPHICS)
                && physical_device
                    .surface_support(i as u32, &surface)
                    .unwrap_or(false)
        })
        .ok_or_else(|| {
            Error::GraphicsInitialization("No suitable queue family found".to_string())
        })? as u32;

    let (device, queues) = Device::new(
        physical_device.clone(),
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
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

/// Create swapchain for window rendering
pub fn create_swapchain(
    device: Arc<Device>,
    surface: Arc<Surface>,
    window_size: [u32; 2],
) -> Result<(Arc<Swapchain>, Vec<Arc<Image>>), Error> {
    let capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to get surface capabilities: {}", e))
        })?;

    let composite_alpha = capabilities
        .supported_composite_alpha
        .into_iter()
        .next()
        .ok_or_else(|| {
            Error::GraphicsInitialization("No composite alpha mode available".to_string())
        })?;

    let surface_format = *device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to get surface formats: {}", e))
        })?
        .iter()
        .next()
        .ok_or_else(|| Error::GraphicsInitialization("No surface formats available".to_string()))?;

    let (swapchain, images) = Swapchain::new(
        device.clone(),
        surface,
        SwapchainCreateInfo {
            min_image_count: capabilities.min_image_count + 1,
            image_format: surface_format.0,
            image_extent: window_size.into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha,
            ..Default::default()
        },
    )
    .map_err(|e| Error::GraphicsInitialization(format!("Failed to create swapchain: {}", e)))?;

    Ok((swapchain, images))
}
