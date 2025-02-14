mod pipeline;
mod renderer;
mod shader;
mod vertex;

pub use pipeline::RenderPipeline;
pub use renderer::Renderer;
pub use shader::{ShaderModule, ShaderType};
pub use vertex::{Vertex, Vertex2D};

use crate::core::Error;
use crate::core::Window;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator};

/// The main graphics context
pub struct Graphics {
    device: Arc<Device>,
    queue: Arc<Queue>,
    renderer: Renderer,
}

impl Graphics {
    pub fn new(_window: &Window) -> Result<Self, Error> {
        Err(Error::GraphicsInitialization(
            "Graphics initialization not yet implemented".into(),
        ))
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
