use crate::commands::{CommandBuffer, CommandPool};
use crate::error::{Result, VulkanError};
use crate::pipeline::Pipeline;
use crate::render_pass::RenderPass;
use crate::shader::ShaderSet;
use crate::swapchain::Swapchain;
use ash::{vk, Device};
use std::sync::Arc;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Renderer {
    render_pass: RenderPass,
    pipeline: Pipeline,
    command_pool: CommandPool,
    command_buffers: Vec<CommandBuffer>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
    device: Arc<Device>,
}

impl Renderer {
    pub fn new(
        device: Arc<Device>,
        swapchain: &Swapchain,
        shader_set: &ShaderSet,
        queue_family_index: u32,
    ) -> Result<Self> {
        // Create render pass
        let render_pass = RenderPass::new(
            device.clone(),
            swapchain.surface_format().format,
            swapchain.image_views(),
            swapchain.extent(),
        )?;

        // Create graphics pipeline
        let shader_stages = shader_set.create_shader_stages();
        let pipeline = Pipeline::new(
            device.clone(),
            render_pass.handle(),
            &shader_stages,
            swapchain.extent(),
        )?;

        // Create command pool and buffers
        let command_pool = CommandPool::new(
            device.clone(),
            queue_family_index,
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        )?;

        let command_buffers = command_pool.allocate_buffers(
            vk::CommandBufferLevel::PRIMARY,
            swapchain.image_views().len() as u32,
        )?;

        // Create synchronization primitives
        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                let image_available = device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| VulkanError::SyncCreation(e.to_string()))?;
                let render_finished = device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| VulkanError::SyncCreation(e.to_string()))?;
                let in_flight = device
                    .create_fence(&fence_info, None)
                    .map_err(|e| VulkanError::SyncCreation(e.to_string()))?;

                image_available_semaphores.push(image_available);
                render_finished_semaphores.push(render_finished);
                in_flight_fences.push(in_flight);
            }
        }

        Ok(Self {
            render_pass,
            pipeline,
            command_pool,
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
            device,
        })
    }

    pub fn render_frame(
        &mut self,
        swapchain: &Swapchain,
        graphics_queue: vk::Queue,
        present_queue: vk::Queue,
    ) -> Result<bool> {
        let image_available = self.image_available_semaphores[self.current_frame];
        let render_finished = self.render_finished_semaphores[self.current_frame];
        let in_flight = self.in_flight_fences[self.current_frame];

        unsafe {
            self.device
                .wait_for_fences(&[in_flight], true, u64::MAX)
                .map_err(|e| VulkanError::SyncError(e.to_string()))?;
        }

        let (image_index, is_suboptimal) =
            match swapchain.acquire_next_image(image_available, vk::Fence::null()) {
                Ok(result) => result,
                Err(VulkanError::SwapchainCreation(_)) => return Ok(true), // Swapchain needs recreation
                Err(e) => return Err(e),
            };

        unsafe {
            self.device
                .reset_fences(&[in_flight])
                .map_err(|e| VulkanError::SyncError(e.to_string()))?;
        }

        // Record command buffer
        let command_buffer = &mut self.command_buffers[image_index as usize];
        command_buffer.reset(false)?;

        {
            let mut recorder = command_buffer.begin(vk::CommandBufferUsageFlags::empty())?;

            self.render_pass.begin_render_pass(
                recorder.handle(),
                image_index as usize,
                swapchain.extent(),
                [0.0, 0.0, 0.0, 1.0],
            );

            self.pipeline.bind(recorder.handle());
            recorder.draw(3, 1, 0, 0); // Draw a triangle
            recorder.end_render_pass();
            recorder.end()?;
        }

        // Submit command buffer
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        command_buffer.submit(
            graphics_queue,
            &[image_available],
            &wait_stages,
            &[render_finished],
            in_flight,
        )?;

        // Present the frame
        if let Err(VulkanError::SwapchainCreation(_)) =
            swapchain.present(present_queue, image_index, &[render_finished])
        {
            return Ok(true); // Swapchain needs recreation
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(is_suboptimal)
    }

    pub fn wait_idle(&self) -> Result<()> {
        unsafe {
            self.device
                .device_wait_idle()
                .map_err(|e| VulkanError::SyncError(e.to_string()))?;
        }
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // Wait for the device to finish all operations
        unsafe {
            if let Ok(_) = self.device.device_wait_idle() {
                for &semaphore in &self.image_available_semaphores {
                    self.device.destroy_semaphore(semaphore, None);
                }
                for &semaphore in &self.render_finished_semaphores {
                    self.device.destroy_semaphore(semaphore, None);
                }
                for &fence in &self.in_flight_fences {
                    self.device.destroy_fence(fence, None);
                }
            }
        }
    }
}
