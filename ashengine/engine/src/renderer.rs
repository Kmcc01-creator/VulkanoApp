use crate::error::{Result, VulkanError};
use crate::{pipeline::Pipeline, render_pass::RenderPass, swapchain::Swapchain};
use ash::{vk, Device};
use std::sync::Arc;

pub struct Renderer {
    device: Arc<Device>,
    swapchain: Option<Swapchain>,
    render_pass: Option<RenderPass>,
    pipeline: Option<Pipeline>,
    command_buffers: Vec<vk::CommandBuffer>,
    command_pool: vk::CommandPool,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
    frames_in_flight: usize,
    graphics_queue: vk::Queue,
    current_image_index: Option<u32>,
}

impl Renderer {
    pub fn new(
        device: Arc<Device>,
        graphics_queue: vk::Queue,
        queue_family_index: u32,
    ) -> Result<Self> {
        let frames_in_flight = 2;

        // Create command pool
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        let command_pool = unsafe {
            device
                .create_command_pool(&pool_info, None)
                .map_err(|e| VulkanError::CommandPoolCreation(e.to_string()))?
        };

        // Create synchronization objects
        let mut image_available_semaphores = Vec::with_capacity(frames_in_flight);
        let mut render_finished_semaphores = Vec::with_capacity(frames_in_flight);
        let mut in_flight_fences = Vec::with_capacity(frames_in_flight);
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..frames_in_flight {
            unsafe {
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| VulkanError::SemaphoreCreation(e.to_string()))?;

                let render_finished_semaphore = device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| VulkanError::SemaphoreCreation(e.to_string()))?;

                let in_flight_fence = device
                    .create_fence(&fence_info, None)
                    .map_err(|e| VulkanError::FenceCreation(e.to_string()))?;

                image_available_semaphores.push(image_available_semaphore);
                render_finished_semaphores.push(render_finished_semaphore);
                in_flight_fences.push(in_flight_fence);
            }
        }

        Ok(Self {
            device,
            swapchain: None,
            render_pass: None,
            pipeline: None,
            command_buffers: Vec::new(),
            command_pool,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
            frames_in_flight,
            graphics_queue,
            current_image_index: None,
        })
    }

    pub fn begin_frame(&mut self) -> Result<()> {
        let fence = self.in_flight_fences[self.current_frame];

        unsafe {
            self.device
                .wait_for_fences(&[fence], true, u64::MAX)
                .map_err(|e| VulkanError::SyncError(e.to_string()))?;

            self.device
                .reset_fences(&[fence])
                .map_err(|e| VulkanError::SyncError(e.to_string()))?;

            // Reset command buffer
            if !self.command_buffers.is_empty() {
                self.device
                    .reset_command_buffer(
                        self.command_buffers[self.current_frame],
                        vk::CommandBufferResetFlags::empty(),
                    )
                    .map_err(|e| VulkanError::General(e.to_string()))?;
            }
        }

        if let Some(swapchain) = &self.swapchain {
            let (image_index, _) = swapchain.acquire_next_image(
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            )?;
            self.current_image_index = Some(image_index);

            if !self.command_buffers.is_empty() {
                // Begin command buffer
                let command_buffer = self.command_buffers[self.current_frame];
                let begin_info = vk::CommandBufferBeginInfo::builder()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

                unsafe {
                    self.device
                        .begin_command_buffer(command_buffer, &begin_info)
                        .map_err(|e| VulkanError::General(e.to_string()))?;
                }

                // Begin render pass
                if let Some(render_pass) = &self.render_pass {
                    render_pass.begin_render_pass(
                        command_buffer,
                        image_index as usize,
                        swapchain.extent(),
                        [0.0, 0.0, 0.0, 1.0],
                    );

                    // Bind pipeline and draw triangle
                    if let Some(pipeline) = &self.pipeline {
                        pipeline.bind(command_buffer);
                        unsafe {
                            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<()> {
        if let (Some(swapchain), Some(image_index)) = (&self.swapchain, self.current_image_index) {
            if !self.command_buffers.is_empty() {
                let command_buffer = self.command_buffers[self.current_frame];

                // End render pass
                unsafe {
                    if self.render_pass.is_some() {
                        self.device.cmd_end_render_pass(command_buffer);
                    }

                    // End command buffer
                    self.device
                        .end_command_buffer(command_buffer)
                        .map_err(|e| VulkanError::General(e.to_string()))?;

                    // Submit command buffer
                    let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
                    let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                    let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
                    let command_buffers = [command_buffer];

                    let submit_info = vk::SubmitInfo::builder()
                        .wait_semaphores(&wait_semaphores)
                        .wait_dst_stage_mask(&wait_stages)
                        .command_buffers(&command_buffers)
                        .signal_semaphores(&signal_semaphores);

                    self.device
                        .queue_submit(
                            self.graphics_queue,
                            &[submit_info.build()],
                            self.in_flight_fences[self.current_frame],
                        )
                        .map_err(|e| VulkanError::General(e.to_string()))?;
                }
            }

            swapchain.present(
                self.graphics_queue,
                image_index,
                &[self.render_finished_semaphores[self.current_frame]],
            )?;
        }

        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;
        self.current_image_index = None;
        Ok(())
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn current_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffers[self.current_frame]
    }

    pub fn initialize_swapchain(
        &mut self,
        swapchain: Swapchain,
        render_pass: RenderPass,
        vert_shader: &[u8],
        frag_shader: &[u8],
    ) -> Result<()> {
        // Create command buffers
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(self.frames_in_flight as u32);

        let command_buffers = unsafe {
            self.device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .map_err(|e| VulkanError::CommandBufferAllocation(e.to_string()))?
        };

        // Create pipeline
        let pipeline = Pipeline::new(
            self.device.clone(),
            render_pass.handle(),
            swapchain.extent(),
            vert_shader,
            frag_shader,
        )?;

        self.pipeline = Some(pipeline);
        self.command_buffers = command_buffers;
        self.swapchain = Some(swapchain);
        self.render_pass = Some(render_pass);
        Ok(())
    }

    pub fn image_available_semaphore(&self) -> vk::Semaphore {
        self.image_available_semaphores[self.current_frame]
    }

    pub fn render_finished_semaphore(&self) -> vk::Semaphore {
        self.render_finished_semaphores[self.current_frame]
    }

    pub fn in_flight_fence(&self) -> vk::Fence {
        self.in_flight_fences[self.current_frame]
    }

    pub fn set_command_buffers(&mut self, command_buffers: Vec<vk::CommandBuffer>) {
        self.command_buffers = command_buffers;
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
            for semaphore in &self.image_available_semaphores {
                self.device.destroy_semaphore(*semaphore, None);
            }
            for semaphore in &self.render_finished_semaphores {
                self.device.destroy_semaphore(*semaphore, None);
            }
            for fence in &self.in_flight_fences {
                self.device.destroy_fence(*fence, None);
            }
        }
    }
}
