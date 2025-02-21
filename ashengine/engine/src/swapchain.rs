use crate::error::{Result, VulkanError};
use ash::extensions::khr;
use ash::{vk, Device, Instance};
use std::sync::Arc;

pub struct Swapchain {
    swapchain_loader: khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    surface_format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    device: Arc<Device>,
    instance: Arc<Instance>,
    surface_loader: Arc<khr::Surface>,
}

impl Swapchain {
    pub fn new(
        physical_device: vk::PhysicalDevice,
        device: Arc<Device>,
        instance: Arc<Instance>,
        surface_loader: Arc<khr::Surface>,
        surface: vk::SurfaceKHR,
        dimensions: [u32; 2],
    ) -> Result<Self> {
        // Get surface capabilities
        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))?
        };

        // Check for zero size
        if capabilities.current_extent.width == 0 || capabilities.current_extent.height == 0 {
            return Err(VulkanError::SwapchainCreation(
                "Window surface has zero size".to_string(),
            ));
        }

        // Choose surface format
        let surface_format = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))?
                .into_iter()
                .find(|format| {
                    format.format == vk::Format::B8G8R8A8_SRGB
                        && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                })
                .unwrap_or_else(|| {
                    surface_loader
                        .get_physical_device_surface_formats(physical_device, surface)
                        .unwrap()[0]
                })
        };

        // Choose present mode (prefer mailbox/triple buffering if available)
        let present_mode = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))?
                .into_iter()
                .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
                .unwrap_or(vk::PresentModeKHR::FIFO)
        };

        // Calculate extent
        let extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: dimensions[0].clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: dimensions[1].clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        };

        // Calculate image count - ensure it's at least the minimum required
        let image_count = capabilities.min_image_count + 1;
        let image_count = if capabilities.max_image_count > 0 {
            image_count.min(capabilities.max_image_count)
        } else {
            image_count
        };

        // Create swapchain
        let swapchain_loader = khr::Swapchain::new(&instance, &device);
        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))?
        };

        // Get swapchain images
        let images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))?
        };

        // Create image views
        let image_views: Vec<_> = images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
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
                unsafe {
                    device
                        .create_image_view(&create_info, None)
                        .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            swapchain_loader,
            swapchain,
            images,
            image_views,
            surface_format,
            extent,
            present_mode,
            device,
            instance,
            surface_loader,
        })
    }

    pub fn acquire_next_image(
        &self,
        semaphore: vk::Semaphore,
        fence: vk::Fence,
    ) -> Result<(u32, bool)> {
        match unsafe {
            self.swapchain_loader
                .acquire_next_image(self.swapchain, u64::MAX, semaphore, fence)
        } {
            Ok((index, suboptimal)) => {
                if suboptimal {
                    Err(VulkanError::SwapchainSuboptimal)
                } else {
                    Ok((index, false))
                }
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Err(VulkanError::SwapchainOutOfDate),
            Err(e) => Err(VulkanError::SwapchainCreation(e.to_string())),
        }
    }

    pub fn present(
        &self,
        queue: vk::Queue,
        image_index: u32,
        wait_semaphores: &[vk::Semaphore],
    ) -> Result<bool> {
        let swapchains = [self.swapchain];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        match unsafe { self.swapchain_loader.queue_present(queue, &present_info) } {
            Ok(suboptimal) => {
                if suboptimal {
                    Err(VulkanError::SwapchainSuboptimal)
                } else {
                    Ok(false)
                }
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Err(VulkanError::SwapchainOutOfDate),
            Err(e) => Err(VulkanError::SwapchainCreation(e.to_string())),
        }
    }

    pub fn surface_format(&self) -> vk::SurfaceFormatKHR {
        self.surface_format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn image_views(&self) -> &[vk::ImageView] {
        &self.image_views
    }

    pub fn recreate(
        &mut self,
        physical_device: vk::PhysicalDevice,
        device: Arc<Device>,
        instance: Arc<Instance>,
        surface_loader: Arc<khr::Surface>,
        surface: vk::SurfaceKHR,
        dimensions: [u32; 2],
    ) -> Result<()> {
        // Wait for the device to be idle before recreating
        unsafe {
            self.device
                .device_wait_idle()
                .map_err(|e| VulkanError::SyncError(e.to_string()))?;
        }

        // Clean up old image views
        unsafe {
            for &image_view in &self.image_views {
                self.device.destroy_image_view(image_view, None);
            }
        }

        // Get new surface capabilities
        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))?
        };

        // Check for zero size
        if capabilities.current_extent.width == 0 || capabilities.current_extent.height == 0 {
            return Err(VulkanError::SwapchainCreation(
                "Window surface has zero size".to_string(),
            ));
        }

        // Calculate new extent
        let new_extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: dimensions[0].clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: dimensions[1].clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        };

        // Create new swapchain
        let swapchain_loader = khr::Swapchain::new(&self.instance, &self.device);
        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(capabilities.min_image_count + 1)
            .image_format(self.surface_format.format)
            .image_color_space(self.surface_format.color_space)
            .image_extent(new_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(self.present_mode)
            .clipped(true)
            .old_swapchain(self.swapchain);

        let new_swapchain = unsafe {
            self.swapchain_loader
                .create_swapchain(&create_info, None)
                .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))?
        };

        // Clean up old swapchain
        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }

        self.swapchain = new_swapchain;
        self.extent = new_extent;
        self.device = device;
        self.instance = instance;
        self.surface_loader = surface_loader;
        self.swapchain_loader = swapchain_loader;

        // Get new swapchain images and create new image views
        let images = unsafe {
            self.swapchain_loader
                .get_swapchain_images(self.swapchain)
                .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))?
        };

        let image_views: Vec<_> = images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(self.surface_format.format)
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
                unsafe {
                    self.device
                        .create_image_view(&create_info, None)
                        .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        self.images = images;
        self.image_views = image_views;

        Ok(())
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for &image_view in &self.image_views {
                self.device.destroy_image_view(image_view, None);
            }
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }
}
