use crate::error::{Result, VulkanError};
use crate::{
    lighting::Lighting,
    physics::{PhysicsObject, PhysicsWorld},
    pipeline::Pipeline,
    render_pass::RenderPass,
    shader::ShaderSet,
    swapchain::Swapchain,
};
use ash::{vk, Device, Instance};
use glam::Vec3;
use std::sync::Arc;

fn extent_to_array(extent: vk::Extent2D) -> [u32; 2] {
    [extent.width, extent.height]
}

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
    #[allow(dead_code)]
    physical_device: vk::PhysicalDevice,
    #[allow(dead_code)]
    instance: Arc<Instance>,
    #[allow(dead_code)]
    surface_loader: Arc<ash::extensions::khr::Surface>,
    surface: vk::SurfaceKHR,
    shader_set: ShaderSet,
    descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
    physics_world: PhysicsWorld,
    lighting: Lighting,
}

impl Renderer {
    pub fn new(
        device: Arc<Device>,
        graphics_queue: vk::Queue,
        queue_family_index: u32,
        physical_device: vk::PhysicalDevice,
        instance: Arc<Instance>,
        surface_loader: Arc<ash::extensions::khr::Surface>,
        surface: vk::SurfaceKHR,
        shader_set: ShaderSet,
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
    ) -> Result<Self> {
        let frames_in_flight = 2;
        log::debug!(
            "Creating renderer with {} frames in flight",
            frames_in_flight
        );

        // Create command pool
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        let command_pool = unsafe {
            device
                .create_command_pool(&pool_info, None)
                .map_err(|e| VulkanError::CommandPoolCreation(e.to_string()))?
        };
        log::debug!("Command pool created successfully");

        // Create synchronization objects
        let mut image_available_semaphores = Vec::with_capacity(frames_in_flight);
        let mut render_finished_semaphores = Vec::with_capacity(frames_in_flight);
        let mut in_flight_fences = Vec::with_capacity(frames_in_flight);
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for i in 0..frames_in_flight {
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
            log::debug!("Created synchronization objects for frame {}", i);
        }

        // Initialize physics world
        let mut physics_world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0)); // Example gravity

        // Add some placeholder objects
        physics_world.add_object(PhysicsObject {
            position: Vec3::new(0.0, 10.0, 0.0),
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            mass: 1.0,
            bounding_box: Vec4::new(0.0, 0.0, 0.0, 1.0), // Assuming a unit cube for now
        });
        physics_world.add_object(PhysicsObject {
            position: Vec3::new(2.0, 15.0, 0.0),
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            mass: 2.0,
            bounding_box: Vec4::new(0.0, 0.0, 0.0, 0.5), // Assuming a unit cube for now
        });

        // Initialize lighting
        let lighting = Lighting {
            ambient_color: Vec3::new(1.0, 1.0, 1.0),
            ambient_intensity: 0.2,
            directional_lights: vec![],
        };

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
            physical_device,
            instance,
            surface_loader,
            surface,
            shader_set,
            descriptor_set_layouts: descriptor_set_layouts.to_vec(),
            physics_world,
            lighting,
        })
    }

    pub fn handle_resize(&mut self, dimensions: [u32; 2]) -> Result<()> {
        log::debug!("Handling resize to dimensions: {:?}", dimensions);

        unsafe {
            self.device
                .device_wait_idle()
                .map_err(|e| VulkanError::SyncError(e.to_string()))?;
        }

        if let Some(swapchain) = &mut self.swapchain {
            if dimensions[0] == 0 || dimensions[1] == 0 {
                log::debug!("Skipping resize due to zero dimensions");
                return Ok(());
            }

            log::debug!("Recreating swapchain");
            swapchain.recreate(dimensions[0], dimensions[1], self.surface)?;

            // Recreate the render pass with the new swapchain's format and image views
            log::debug!("Recreating render pass");
            self.render_pass = Some(RenderPass::new(
                self.device.clone(),
                swapchain.surface_format(),
                swapchain.image_views(),
                swapchain.extent(),
            )?);

            if let Some(render_pass) = &self.render_pass {
                log::debug!("Recreating pipeline");
                let shader_stages = self.shader_set.create_shader_stages();
                self.pipeline = Some(Pipeline::new(
                    self.device.clone(),
                    render_pass.handle(),
                    swapchain.extent(),
                    &shader_stages,
                    &self.descriptor_set_layouts,
                )?);
            }
        }
        log::debug!("Resize handled successfully");
        Ok(())
    }

    pub fn initialize_swapchain(
        &mut self,
        swapchain: Swapchain,
        render_pass: RenderPass,
    ) -> Result<()> {
        log::debug!("Initializing swapchain");

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(self.frames_in_flight as u32);

        let command_buffers = unsafe {
            self.device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .map_err(|e| VulkanError::CommandBufferAllocation(e.to_string()))?
        };

        log::debug!("Creating graphics pipeline");
        let shader_stages = self.shader_set.create_shader_stages();
        let pipeline = Pipeline::new(
            self.device.clone(),
            render_pass.handle(),
            swapchain.extent(),
            &shader_stages,
            &self.descriptor_set_layouts,
        )?;

        self.pipeline = Some(pipeline);
        self.command_buffers = command_buffers;
        self.swapchain = Some(swapchain);
        self.render_pass = Some(render_pass);
        log::debug!("Swapchain initialization complete");
        Ok(())
    }

    pub fn begin_frame(&mut self) -> Result<()> {
        log::debug!("Beginning frame {}", self.current_frame);
        let fence = self.in_flight_fences[self.current_frame];

        unsafe {
            log::debug!("Waiting for fence");
            self.device
                .wait_for_fences(&[fence], true, u64::MAX)
                .map_err(|e| VulkanError::SyncError(e.to_string()))?;

            self.device
                .reset_fences(&[fence])
                .map_err(|e| VulkanError::SyncError(e.to_string()))?;
        }

        if !self.command_buffers.is_empty() {
            log::debug!("Resetting command buffer");
            unsafe {
                self.device
                    .reset_command_buffer(
                        self.command_buffers[self.current_frame],
                        vk::CommandBufferResetFlags::empty(),
                    )
                    .map_err(|e| VulkanError::General(e.to_string()))?;
            }
        }

        if let Some(swapchain) = &self.swapchain {
            log::debug!("Acquiring next image");
            match swapchain.acquire_next_image(
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            ) {
                Ok((image_index, _)) => {
                    self.current_image_index = Some(image_index);
                    log::debug!("Acquired image index: {}", image_index);

                    if !self.command_buffers.is_empty() {
                        let command_buffer = self.command_buffers[self.current_frame];
                        let begin_info = vk::CommandBufferBeginInfo::builder()
                            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

                        unsafe {
                            log::debug!("Beginning command buffer recording");
                            self.device
                                .begin_command_buffer(command_buffer, &begin_info)
                                .map_err(|e| VulkanError::General(e.to_string()))?;
                        }

                        if let Some(render_pass) = &self.render_pass {
                            log::debug!("Beginning render pass");
                            render_pass.begin_render_pass(
                                command_buffer,
                                image_index as usize,
                                swapchain.extent(),
                                [0.0, 0.0, 0.0, 1.0],
                            );
                            log::debug!("Render pass started successfully");

                            if let Some(pipeline) = &self.pipeline {
                                log::debug!("Binding pipeline for drawing");
                                pipeline.bind(command_buffer);
                            } else {
                                log::warn!("No pipeline available for drawing");
                            }
                        } else {
                            log::warn!("No render pass available");
                        }
                    }
                }
                Err(VulkanError::SwapchainOutOfDate) => {
                    log::info!("Swapchain out of date, recreating");
                    if let Some(swapchain) = &self.swapchain {
                        self.handle_resize(extent_to_array(swapchain.extent()))?;
                    }
                    return Ok(());
                }
                Err(VulkanError::SwapchainSuboptimal) => {
                    log::info!("Swapchain suboptimal, continuing with frame");
                }
                Err(e) => return Err(e),
            }
        } else {
            log::warn!("No swapchain available");
        }

        // Update physics world
        self.physics_world.update(1.0 / 60.0); // Fixed delta time for now

        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<()> {
        log::debug!("Ending frame {}", self.current_frame);

        if let (Some(swapchain), Some(image_index)) = (&self.swapchain, self.current_image_index) {
            if !self.command_buffers.is_empty() {
                let command_buffer = self.command_buffers[self.current_frame];

                unsafe {
                    if self.render_pass.is_some() {
                        log::debug!("Ending render pass");
                        self.device.cmd_end_render_pass(command_buffer);
                        log::debug!("Render pass ended successfully");
                    }

                    log::debug!("Ending command buffer recording");
                    self.device
                        .end_command_buffer(command_buffer)
                        .map_err(|e| VulkanError::General(e.to_string()))?;

                    log::debug!("Submitting command buffer");
                    let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
                    let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                    let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
                    let command_buffers = [command_buffer];

                    let submit_info = vk::SubmitInfo::builder()
                        .wait_semaphores(&wait_semaphores)
                        .wait_dst_stage_mask(&wait_stages)
                        .command_buffers(&command_buffers)
                        .signal_semaphores(&signal_semaphores);

                    log::debug!("Submitting to graphics queue");
                    self.device
                        .queue_submit(
                            self.graphics_queue,
                            &[submit_info.build()],
                            self.in_flight_fences[self.current_frame],
                        )
                        .map_err(|e| VulkanError::General(e.to_string()))?;
                    log::debug!("Command buffer submitted successfully");
                }
            }

            log::debug!("Presenting frame");
            match swapchain.present(
                self.graphics_queue,
                image_index,
                &[self.render_finished_semaphores[self.current_frame]],
            ) {
                Ok(_) => {
                    log::debug!("Frame presented successfully");
                }
                Err(VulkanError::SwapchainOutOfDate) => {
                    log::info!("Swapchain out of date during present, recreating");
                    self.handle_resize(extent_to_array(swapchain.extent()))?;
                }
                Err(VulkanError::SwapchainSuboptimal) => {
                    log::info!("Swapchain suboptimal during present, continuing");
                }
                Err(e) => return Err(e),
            }
        }

        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;
        self.current_image_index = None;
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

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn current_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffers[self.current_frame]
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline.as_ref().map_or_else(
            || panic!("Pipeline not initialized"),
            |pipeline| pipeline.layout(),
        )
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            log::debug!("Cleaning up renderer resources");
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
            log::debug!("Renderer cleanup complete");
        }
    }
}
