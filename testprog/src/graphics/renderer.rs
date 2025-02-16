use std::sync::Arc;
use vulkano::command_buffer::{
    allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassContents, SubpassEndInfo,
};
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::sync::{self, GpuFuture};

use super::swapchain::SwapchainContext;
use super::vertex::{Vertex2D, Vertex3D};
use super::RenderPipeline;
use crate::core::Error;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct RenderContext {
    render_pass: Arc<RenderPass>,
    graphics_queue: Arc<Queue>,
    swapchain: SwapchainContext,
    framebuffers: Vec<Arc<Framebuffer>>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    current_command_buffer: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    current_frame: usize,
    current_image: u32,
}

impl RenderContext {
    pub fn new(
        render_pass: Arc<RenderPass>,
        graphics_queue: Arc<Queue>,
        swapchain: SwapchainContext,
    ) -> Result<Self, Error> {
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            graphics_queue.device().clone(),
            Default::default(),
        ));

        let framebuffers = create_framebuffers(&swapchain, &render_pass)?;

        Ok(Self {
            render_pass,
            graphics_queue,
            swapchain,
            framebuffers,
            command_buffer_allocator,
            current_command_buffer: None,
            previous_frame_end: Some(sync::now(graphics_queue.device().clone()).boxed()),
            current_frame: 0,
            current_image: 0,
        })
    }

    pub fn begin_frame(&mut self) -> Result<(), Error> {
        // Wait for previous frame
        if let Some(future) = self.previous_frame_end.as_mut() {
            future.cleanup_finished();
        }

        let (image_index, suboptimal, mut acquire_future) =
            match self.swapchain.acquire_next_image(None) {
                Ok((index, suboptimal, future)) => (index, suboptimal, future),
                Err(e) => {
                    return Err(Error::RenderError(format!(
                        "Failed to acquire next image: {}",
                        e
                    )))
                }
            };

        if suboptimal {
            self.recreate_swapchain()?;
        }

        self.current_image = image_index;

        let mut command_buffer = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| Error::RenderError(format!("Failed to create command buffer: {}", e)))?;

        command_buffer
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone(),
                    )
                },
                SubpassContents::Inline,
            )
            .map_err(|e| Error::RenderError(format!("Failed to begin render pass: {}", e)))?;

        self.current_command_buffer = Some(command_buffer);
        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<(), Error> {
        let mut command_buffer = self
            .current_command_buffer
            .take()
            .ok_or_else(|| Error::RenderError("No command buffer to submit".to_string()))?;

        command_buffer
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| Error::RenderError(format!("Failed to end render pass: {}", e)))?;

        let command_buffer = command_buffer
            .build()
            .map_err(|e| Error::RenderError(format!("Failed to build command buffer: {}", e)))?;

        let previous_future = self
            .previous_frame_end
            .take()
            .unwrap_or_else(|| sync::now(self.graphics_queue.device().clone()).boxed());

        let future = previous_future
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .map_err(|e| Error::RenderError(format!("Failed to execute command buffer: {}", e)))?
            .then_swapchain_present(
                self.graphics_queue.clone(),
                self.swapchain.present_info(self.current_image),
            )
            .then_signal_fence_and_flush()
            .map_err(|e| Error::RenderError(format!("Failed to flush future: {}", e)))?;

        self.previous_frame_end = Some(Box::new(future));
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    pub fn recreate_swapchain(&mut self) -> Result<(), Error> {
        self.swapchain.recreate()?;
        self.framebuffers = create_framebuffers(&self.swapchain, &self.render_pass)?;
        Ok(())
    }

    pub fn current_command_buffer(
        &mut self,
    ) -> Option<&mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>> {
        self.current_command_buffer.as_mut()
    }
}

fn create_framebuffers(
    swapchain: &SwapchainContext,
    render_pass: &Arc<RenderPass>,
) -> Result<Vec<Arc<Framebuffer>>, Error> {
    swapchain
        .images()
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to create image view: {}", e))
            })?;

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to create framebuffer: {}", e))
            })
        })
        .collect()
}
