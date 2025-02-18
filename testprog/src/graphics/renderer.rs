use std::sync::Arc;
use vulkano::command_buffer::{
    allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
    SubpassEndInfo,
};
use vulkano::device::Queue;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::{self, SwapchainPresentInfo};
use vulkano::sync::{self, GpuFuture};

use super::swapchain::SwapchainContext;
use crate::core::error::Error;
use glam::{Vec2, Vec3, Vec4};

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct RenderContext {
    render_pass: Arc<RenderPass>,
    graphics_queue: Arc<Queue>,
    swapchain: SwapchainContext,
    framebuffers: Vec<Arc<Framebuffer>>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    current_command_buffer: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    current_frame: usize,
    current_image: u32,
    camera: Camera,
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub view_matrix: glam::Mat4,
    pub projection_matrix: glam::Mat4,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            view_matrix: glam::Mat4::IDENTITY,
            projection_matrix: glam::Mat4::perspective_rh(
                std::f32::consts::PI / 4.0,
                1.0,
                0.1,
                1000.0,
            ),
        }
    }
}

impl RenderContext {
    pub fn new(graphics_queue: Arc<Queue>, swapchain: SwapchainContext) -> Result<Self, Error> {
        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            graphics_queue.device().clone(),
            Default::default(),
        );

        let render_pass = vulkano::single_pass_renderpass!(
            graphics_queue.device().clone(),
            attachments: {
                color: {
                    format: swapchain.format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth_stencil: {
                    format: vulkano::format::Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth_stencil}
            }
        )
        .map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to create render pass: {}", e))
        })?;

        let framebuffers = create_framebuffers(&swapchain, &render_pass)?;

        Ok(Self {
            render_pass,
            graphics_queue: graphics_queue.clone(),
            swapchain,
            framebuffers,
            command_buffer_allocator,
            current_command_buffer: None,
            previous_frame_end: Some(sync::now(graphics_queue.device().clone()).boxed()),
            current_frame: 0,
            current_image: 0,
            camera: Camera::default(),
        })
    }

    // Drawing methods
    pub fn draw_rect(&mut self, size: Vec2, position: Vec2, color: Vec4) -> Result<(), Error> {
        // TODO: Implement actual rectangle drawing using Vulkan pipeline
        let _rect_data = format!(
            "Draw rect at ({}, {}) size ({}, {}) color ({:?})",
            position.x, position.y, size.x, size.y, color
        );
        Ok(())
    }

    pub fn draw_rect_outline(
        &mut self,
        size: Vec2,
        position: Vec2,
        width: f32,
        color: Vec4,
    ) -> Result<(), Error> {
        // TODO: Implement rectangle outline drawing
        let _outline_data = format!(
            "Draw rect outline at ({}, {}) size ({}, {}) width {} color ({:?})",
            position.x, position.y, size.x, size.y, width, color
        );
        Ok(())
    }

    pub fn draw_text(&mut self, text: &str, position: Vec2, color: Vec4) -> Result<(), Error> {
        // TODO: Implement text rendering
        let _text_data = format!(
            "Draw text '{}' at ({}, {}) color ({:?})",
            text, position.x, position.y, color
        );
        Ok(())
    }

    pub fn measure_text(&self, text: &str) -> Vec2 {
        // TODO: Implement actual text measurement
        Vec2::new(text.len() as f32 * 8.0, 14.0) // Temporary approximation
    }

    pub fn world_to_screen(&self, world_pos: Vec3) -> Option<Vec2> {
        let clip_pos =
            self.camera.projection_matrix * self.camera.view_matrix * world_pos.extend(1.0);
        if clip_pos.w <= 0.0 {
            return None;
        }

        let ndc = clip_pos.truncate() / clip_pos.w;

        let [width, height] = self.swapchain.extent();
        let viewport_size = Vec2::new(width as f32, height as f32);

        Some(Vec2::new(
            (ndc.x + 1.0) * viewport_size.x * 0.5,
            (1.0 - ndc.y) * viewport_size.y * 0.5,
        ))
    }

    // Frame management
    pub fn begin_frame(&mut self) -> Result<(), Error> {
        if let Some(future) = self.previous_frame_end.as_mut() {
            future.cleanup_finished();
        }

        let (image_index, suboptimal, _acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
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

        let clear_values = vec![Some([0.0, 0.0, 0.0, 1.0].into()), Some(1.0.into())];

        let builder = {
            let mut builder = AutoCommandBufferBuilder::primary(
                &self.command_buffer_allocator,
                self.graphics_queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .map_err(|e| Error::RenderError(format!("Failed to create command buffer: {}", e)))?;

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values,
                        ..RenderPassBeginInfo::framebuffer(
                            self.framebuffers[image_index as usize].clone(),
                        )
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .map_err(|e| Error::RenderError(format!("Failed to begin render pass: {}", e)))?;

            builder
        };

        self.current_command_buffer = Some(builder);
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

        let info =
            SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), self.current_image);

        let future = previous_future
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .map_err(|e| Error::RenderError(format!("Failed to execute command buffer: {}", e)))?
            .then_swapchain_present(self.graphics_queue.clone(), info)
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

    pub fn viewport_size(&self) -> Vec2 {
        let [width, height] = self.swapchain.extent();
        Vec2::new(width as f32, height as f32)
    }
}

fn create_framebuffers(
    swapchain: &SwapchainContext,
    render_pass: &Arc<RenderPass>,
) -> Result<Vec<Arc<Framebuffer>>, Error> {
    swapchain
        .image_views()
        .iter()
        .map(|view| {
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view.clone()],
                    ..Default::default()
                },
            )
            .map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to create framebuffer: {}", e))
            })
        })
        .collect()
}
