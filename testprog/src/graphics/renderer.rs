use std::sync::Arc;
use vulkano::command_buffer::{
    allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo,
};
use vulkano::device::Queue;
use vulkano::image::view::ImageView;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::swapchain::{
    self, acquire_next_image, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::sync::semaphore::{Semaphore, SemaphoreCreateInfo};
use vulkano::sync::{self, future::GpuFuture};

use super::vertex::{Vertex2D, Vertex3D};
use super::RenderPipeline;
use crate::core::Error;

type CommandBuilder = AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>;
type SwapchainImageView = ImageView<swapchain::Image>;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

struct FrameSync {
    image_available: Arc<Semaphore>,
    render_finished: Arc<Semaphore>,
}

pub struct Renderer {
    render_pass: Arc<RenderPass>,
    graphics_queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    swapchain_images: Vec<Arc<Image>>,
    framebuffers: Vec<Arc<Framebuffer>>,
    frame_sync: Vec<FrameSync>,
    current_frame: usize,
    current_image: u32,
    command_buffer_allocator: StandardCommandBufferAllocator,
    current_command_buffer: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl Renderer {
    pub fn new(
        render_pass: Arc<RenderPass>,
        graphics_queue: Arc<Queue>,
        swapchain: Arc<Swapchain>,
    ) -> Result<Self, Error> {
        // Create command buffer allocator
        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            graphics_queue.device().clone(),
            Default::default(),
        );

        // Get swapchain images directly from the swapchain
        let swapchain_images = swapchain
            .image_views(ImageView::new_default)
            .map_err(|e| Error::GraphicsInitialization(e.to_string()))?;

        // Create framebuffers for each swapchain image view
        let framebuffers = swapchain_images
            .into_iter()
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone())
                    .map_err(|e| Error::GraphicsInitialization(e.to_string()))?;

                Framebuffer::new(
                    render_pass.clone(),
                    vulkano::render_pass::FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                )
                .map_err(|e| Error::GraphicsInitialization(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Create synchronization primitives
        let frame_sync = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                let image_available = Semaphore::new(
                    graphics_queue.device().clone(),
                    SemaphoreCreateInfo::default(),
                )
                .map_err(|e| Error::GraphicsInitialization(e.to_string()))?;

                let render_finished = Semaphore::new(
                    graphics_queue.device().clone(),
                    SemaphoreCreateInfo::default(),
                )
                .map_err(|e| Error::GraphicsInitialization(e.to_string()))?;

                Ok(FrameSync {
                    image_available: Arc::new(image_available),
                    render_finished: Arc::new(render_finished),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            render_pass,
            graphics_queue,
            swapchain,
            swapchain_images,
            framebuffers,
            frame_sync,
            current_frame: 0,
            current_image: 0,
            command_buffer_allocator,
            current_command_buffer: None,
            previous_frame_end: Some(sync::now(graphics_queue.device().clone()).boxed()),
        })
    }

    pub fn begin_frame(
        &mut self,
    ) -> Result<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, Error> {
        // Wait for the previous frame to finish
        if let Some(future) = self.previous_frame_end.take() {
            future.cleanup_finished();
        }

        // Get the next image
        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok((i, s)) => (
                    i,
                    s,
                    sync::now(self.graphics_queue.device().clone()).boxed(),
                ),
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

        // Create command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| Error::RenderError(format!("Failed to create command buffer: {}", e)))?;

        // Begin render pass
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone(),
                    )
                },
                SubpassBeginInfo::default(),
            )
            .map_err(|e| Error::RenderError(format!("Failed to begin render pass: {}", e)))?;

        self.current_command_buffer = Some(builder.clone());
        Ok(builder)
    }

    pub fn end_frame(&mut self) -> Result<(), Error> {
        // End the render pass
        let mut builder = self
            .current_command_buffer
            .take()
            .ok_or_else(|| Error::RenderError("No command buffer to submit".to_string()))?;

        builder
            .end_render_pass(Default::default())
            .map_err(|e| Error::RenderError(format!("Failed to end render pass: {}", e)))?;

        let command_buffer = builder
            .build()
            .map_err(|e| Error::RenderError(format!("Failed to build command buffer: {}", e)))?;

        // Submit and present
        let execute_future = self
            .previous_frame_end
            .take()
            .unwrap_or_else(|| sync::now(self.graphics_queue.device().clone()).boxed())
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .map_err(|e| Error::RenderError(format!("Failed to execute command buffer: {}", e)))?
            .then_swapchain_present(
                self.graphics_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(),
                    self.current_image,
                ),
            );

        self.previous_frame_end = Some(execute_future.boxed());
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    pub fn draw_mesh(
        &self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        pipeline: &RenderPipeline,
        vertices: &[Vertex3D],
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
