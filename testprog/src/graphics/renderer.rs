use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::device::Queue;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::swapchain::Swapchain;

use super::vertex::Vertex;
use super::RenderPipeline;
use crate::core::Error;

pub struct Renderer {
    render_pass: Arc<RenderPass>,
    graphics_queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    framebuffers: Vec<Arc<Framebuffer>>,
    command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
}

impl Renderer {
    pub fn new(
        render_pass: Arc<RenderPass>,
        graphics_queue: Arc<Queue>,
        swapchain: Arc<Swapchain>,
    ) -> Result<Self, Error> {
        // TODO: Create framebuffers
        let framebuffers = Vec::new();
        let command_buffers = Vec::new();

        Ok(Self {
            render_pass,
            graphics_queue,
            swapchain,
            framebuffers,
            command_buffers,
        })
    }

    pub fn begin_frame(
        &mut self,
    ) -> Result<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, Error> {
        Err(Error::RenderError(
            "Frame beginning not yet implemented".into(),
        ))
    }

    pub fn end_frame(&mut self) -> Result<(), Error> {
        Err(Error::RenderError(
            "Frame submission not yet implemented".into(),
        ))
    }

    pub fn draw_mesh(
        &self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        pipeline: &RenderPipeline,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<(), Error> {
        Err(Error::RenderError(
            "Mesh drawing not yet implemented".into(),
        ))
    }

    pub fn update_viewport(&mut self, width: u32, height: u32) -> Result<(), Error> {
        Err(Error::RenderError(
            "Viewport update not yet implemented".into(),
        ))
    }

    pub fn recreate_swapchain(&mut self) -> Result<(), Error> {
        Err(Error::RenderError(
            "Swapchain recreation not yet implemented".into(),
        ))
    }
}
