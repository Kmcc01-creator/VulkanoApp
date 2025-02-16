use std::sync::Arc;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::{Image, ImageUsage};
use vulkano::swapchain::{
    acquire_next_image, PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::sync::{GpuFuture, Semaphore};

use crate::core::Error;

pub struct SwapchainContext {
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,
    surface_format: Format,
    present_mode: PresentMode,
}

impl SwapchainContext {
    pub fn new(
        device: Arc<Device>,
        surface: Arc<Surface>,
        present_mode: Option<PresentMode>,
    ) -> Result<Self, Error> {
        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to get surface capabilities: {}", e))
            })?;

        let surface_formats = device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to get surface formats: {}", e))
            })?;

        // Choose the first available format
        let surface_format = surface_formats[0].0;

        // Use provided present mode or default to Fifo (vsync)
        let present_mode = present_mode.unwrap_or(PresentMode::Fifo);

        let window = surface
            .object()
            .unwrap()
            .downcast_ref::<winit::window::Window>()
            .unwrap();
        let image_extent: [u32; 2] = window.inner_size().into();

        let (swapchain, images) = Swapchain::new(
            device,
            surface,
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count,
                image_format: surface_format,
                image_extent,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .iter()
                    .next()
                    .unwrap(),
                present_mode,
                ..Default::default()
            },
        )
        .map_err(|e| Error::GraphicsInitialization(format!("Failed to create swapchain: {}", e)))?;

        Ok(Self {
            swapchain,
            images,
            surface_format,
            present_mode,
        })
    }

    pub fn recreate(&mut self) -> Result<(), Error> {
        let window = self
            .swapchain
            .surface()
            .object()
            .unwrap()
            .downcast_ref::<winit::window::Window>()
            .unwrap();
        let image_extent: [u32; 2] = window.inner_size().into();

        let (new_swapchain, new_images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent,
                ..self.swapchain.create_info()
            })
            .map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to recreate swapchain: {}", e))
            })?;

        self.swapchain = new_swapchain;
        self.images = new_images;

        Ok(())
    }

    pub fn acquire_next_image(
        &self,
        semaphore: Arc<Semaphore>,
    ) -> Result<(u32, bool, FenceSignalFuture<Box<dyn GpuFuture>>), Error> {
        acquire_next_image(self.swapchain.clone(), None)
            .map_err(|e| Error::RenderError(format!("Failed to acquire next image: {}", e)))
    }

    pub fn format(&self) -> Format {
        self.surface_format
    }

    pub fn extent(&self) -> [u32; 2] {
        self.swapchain.image_extent()
    }
}
