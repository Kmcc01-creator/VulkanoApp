use crate::error::{Result, VulkanError};
use ash::{vk, Device};
use std::sync::Arc;

pub struct Renderer {
    device: Arc<Device>,
    command_buffers: Vec<vk::CommandBuffer>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
    frames_in_flight: usize,
}

impl Renderer {
    pub fn new(device: Arc<Device>) -> Result<Self> {
        let frames_in_flight = 2;
        let mut image_available_semaphores = Vec::with_capacity(frames_in_flight);
        let mut render_finished_semaphores = Vec::with_capacity(frames_in_flight);
        let mut in_flight_fences = Vec::with_capacity(frames_in_flight);

        // Create synchronization objects
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
            command_buffers: Vec::new(),
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
            frames_in_flight,
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
        }

        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<()> {
        unsafe {
            self.device
                .wait_for_fences(&[self.in_flight_fences[self.current_frame]], true, u64::MAX)
                .map_err(|e| VulkanError::SyncError(e.to_string()))?;
        }

        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;
        Ok(())
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn current_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffers[self.current_frame]
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
