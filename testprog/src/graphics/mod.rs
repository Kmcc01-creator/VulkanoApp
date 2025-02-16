mod pipeline;
mod renderer;
mod shader;
mod vertex;

pub use pipeline::RenderPipeline;
pub use renderer::Renderer;
pub use shader::{ShaderModule, ShaderType};
pub use vertex::Vertex2D;

use crate::core::Error;
use crate::core::Window;
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo,
    },
    format::Format,
    image::{Image, ImageUsage, SwapchainImage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator},
    render_pass::{AttachmentLoadOp, AttachmentStoreOp, LoadOp, RenderPass, StoreOp},
    swapchain::{CompositeAlpha, Surface, Swapchain, SwapchainCreateInfo},
};

use winit::window::Window as WinitWindow;

fn create_swapchain(
    device: Arc<Device>,
    surface: Arc<Surface>,
) -> Result<(Arc<Swapchain>, Vec<Arc<Image>>), Error> {
    let surface_capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to get surface capabilities: {}", e))
        })?;

    let surface_format = device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to get surface format: {}", e))
        })?[0];

    let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
    let window_size = window.inner_size();

    let (swapchain, images) = Swapchain::new(
        device,
        surface,
        SwapchainCreateInfo {
            min_image_count: surface_capabilities.min_image_count.max(2),
            image_format: surface_format.0,
            image_extent: window_size.into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: surface_capabilities
                .supported_composite_alpha
                .iter()
                .next()
                .unwrap(),
            ..Default::default()
        },
    )
    .map_err(|e| Error::GraphicsInitialization(format!("Failed to create swapchain: {}", e)))?;

    Ok((swapchain, images))
}

/// The main graphics context
pub struct Graphics {
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    renderer: Renderer,
}

impl Graphics {
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

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to enumerate devices: {}", e))
            })?
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags
                            .intersects(vulkano::device::QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
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
            .ok_or_else(|| Error::GraphicsInitialization("No suitable GPU found".into()))?;

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

        // Create memory allocator
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        // Get surface format
        let surface_format = device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to get surface format: {}", e))
            })?[0]
            .0;

        // Create render pass
        let render_pass = vulkano::ordered_passes_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: surface_format,
                    samples: 1,
                    load: AttachmentLoadOp::Clear,
                    store: AttachmentStoreOp::Store,
                }
            },
            passes: [
                { color: [color], depth_stencil: {} }
            ]
        )
        .map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to create render pass: {}", e))
        })?;

        // Create swapchain
        let (swapchain, _images) = create_swapchain(device.clone(), surface.clone())?;

        // Create renderer
        let renderer = Renderer::new(render_pass, queue.clone(), swapchain)?;

        Ok(Self {
            instance,
            surface,
            device,
            queue,
            memory_allocator,
            renderer,
        })
    }

    pub fn begin_frame(&mut self) -> Result<(), Error> {
        self.renderer.begin_frame().map(|_| ())
    }

    pub fn end_frame(&mut self) -> Result<(), Error> {
        self.renderer.end_frame()
    }

    pub fn update_viewport(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.renderer.update_viewport(width, height)
    }
}
