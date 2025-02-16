use std::sync::Arc;
use vulkano::device::{
    physical::{PhysicalDevice, PhysicalDeviceType},
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo,
};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::swapchain::Surface;

use crate::core::Error;
use crate::core::Window;

pub struct DeviceContext {
    pub instance: Arc<Instance>,
    pub surface: Arc<Surface>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

impl DeviceContext {
    pub fn new(window: &Window) -> Result<Self, Error> {
        // Create instance with required extensions
        let library = vulkano::VulkanLibrary::new().map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to load Vulkan library: {}", e))
        })?;

        let required_extensions = window.get_required_extensions()?;
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .map_err(|e| Error::GraphicsInitialization(format!("Failed to create instance: {}", e)))?;

        // Create surface
        let surface = window.create_surface(instance.clone())?;

        // Select physical device and queue family
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) =
            Self::select_physical_device(&instance, &surface, &device_extensions)?;

        // Create logical device and queue
        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .map_err(|e| Error::GraphicsInitialization(format!("Failed to create device: {}", e)))?;

        let queue = queues
            .next()
            .ok_or_else(|| Error::GraphicsInitialization("Failed to get queue".into()))?;

        Ok(Self {
            instance,
            surface,
            device,
            queue,
        })
    }

    fn select_physical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface>,
        device_extensions: &DeviceExtensions,
    ) -> Result<(Arc<PhysicalDevice>, u32), Error> {
        instance
            .enumerate_physical_devices()
            .map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to enumerate devices: {}", e))
            })?
            .filter(|p| p.supported_extensions().contains(device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags
                            .intersects(vulkano::device::QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
            .ok_or_else(|| Error::GraphicsInitialization("No suitable GPU found".into()))
    }
}
