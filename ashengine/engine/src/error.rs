use thiserror::Error;

#[derive(Error, Debug)]
pub enum VulkanError {
    #[error("Failed to create instance: {0}")]
    InstanceCreation(String),

    #[error("Failed to create device: {0}")]
    DeviceCreation(String),

    #[error("Failed to create surface: {0}")]
    SurfaceCreation(String),

    #[error("Failed to create swapchain: {0}")]
    SwapchainCreation(String),

    #[error("Failed to create image: {0}")]
    ImageCreation(String),

    #[error("Failed to create image view: {0}")]
    ImageViewCreation(String),

    #[error("Failed to create buffer: {0}")]
    BufferCreation(String),

    #[error("Failed to create render pass: {0}")]
    RenderPassCreation(String),

    #[error("Failed to create pipeline: {0}")]
    PipelineCreation(String),

    #[error("Failed to create pipeline layout: {0}")]
    PipelineLayoutCreation(String),

    #[error("Failed to create shader module: {0}")]
    ShaderCreation(String),

    #[error("Failed to create sampler: {0}")]
    SamplerCreation(String),

    #[error("Failed to create descriptor pool: {0}")]
    DescriptorPoolCreation(String),

    #[error("Failed to create descriptor set layout: {0}")]
    DescriptorSetLayoutCreation(String),

    #[error("Failed to allocate descriptor sets: {0}")]
    DescriptorSetAllocation(String),

    #[error("Failed to create framebuffer: {0}")]
    FramebufferCreation(String),

    #[error("Failed to create command pool: {0}")]
    CommandPoolCreation(String),

    #[error("Failed to allocate command buffers: {0}")]
    CommandBufferAllocation(String),

    #[error("Failed to allocate memory: {0}")]
    MemoryAllocation(String),

    #[error("Failed to bind memory: {0}")]
    MemoryBinding(String),

    #[error("Failed to map memory: {0}")]
    MemoryMapping(String),

    #[error("Failed to create semaphore: {0}")]
    SemaphoreCreation(String),

    #[error("Failed to create fence: {0}")]
    FenceCreation(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Window error: {0}")]
    WindowError(String),

    #[error("Invalid shader file: {0}")]
    InvalidShader(String),

    #[error("General error: {0}")]
    General(String),
}

pub type Result<T> = std::result::Result<T, VulkanError>;
