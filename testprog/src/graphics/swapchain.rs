use crate::core::error::Error;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageUsage};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};

pub struct SwapchainContext {
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    image_views: Vec<Arc<ImageView>>,
    format: Format,
    extent: [u32; 2],
}

impl SwapchainContext {
    pub fn new(device: Arc<Device>, surface: Arc<Surface>) -> Result<Self, Error> {
        let surface_caps = device
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

        let (format, _color_space) = surface_formats
            .iter()
            .next()
            .ok_or_else(|| {
                Error::GraphicsInitialization("No surface formats available".to_string())
            })?
            .clone();

        let extent = surface_caps.current_extent.unwrap_or([800, 600]);

        let composite_alpha = surface_caps
            .supported_composite_alpha
            .into_iter()
            .next()
            .ok_or_else(|| {
                Error::GraphicsInitialization("No composite alpha mode available".to_string())
            })?;

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface,
            SwapchainCreateInfo {
                min_image_count: surface_caps.min_image_count + 1,
                image_format: format,
                image_extent: extent,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                pre_transform: surface_caps.current_transform,
                ..Default::default()
            },
        )
        .map_err(|e| Error::GraphicsInitialization(format!("Failed to create swapchain: {}", e)))?;

        let image_views = images
            .iter()
            .map(|image| {
                ImageView::new_default(image.clone()).map_err(|e| {
                    Error::GraphicsInitialization(format!("Failed to create image view: {}", e))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            swapchain,
            images,
            image_views,
            format,
            extent,
        })
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn extent(&self) -> [u32; 2] {
        self.extent
    }

    pub fn image_views(&self) -> &[Arc<ImageView>] {
        &self.image_views
    }

    pub fn recreate(&mut self) -> Result<(), Error> {
        let create_info = self.swapchain.create_info();
        let (new_swapchain, new_images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: create_info.image_extent,
                image_format: create_info.image_format,
                image_usage: create_info.image_usage,
                composite_alpha: create_info.composite_alpha,
                pre_transform: create_info.pre_transform,
                present_mode: create_info.present_mode,
                clipped: create_info.clipped,
                ..create_info
            })
            .map_err(|e| Error::RenderError(format!("Failed to recreate swapchain: {}", e)))?;

        let new_image_views = new_images
            .iter()
            .map(|image| {
                ImageView::new_default(image.clone()).map_err(|e| {
                    Error::GraphicsInitialization(format!("Failed to create image view: {}", e))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.swapchain = new_swapchain;
        self.images = new_images;
        self.image_views = new_image_views;
        self.extent = create_info.image_extent;

        Ok(())
    }
}

impl std::ops::Deref for SwapchainContext {
    type Target = Arc<Swapchain>;

    fn deref(&self) -> &Self::Target {
        &self.swapchain
    }
}
