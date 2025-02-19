use thiserror::Error;

#[derive(Debug, Error)]
pub enum VulkanError {
    #[error("Failed to create Vulkan instance: {0}")]
    InstanceCreation(String),

    #[error("Failed to create Vulkan device: {0}")]
    DeviceCreation(String),

    #[error("Failed to create Vulkan surface: {0}")]
    SurfaceCreation(String),

    #[error("Failed to create swapchain: {0}")]
    SwapchainCreation(String),

    #[error("Failed to create shader module: {0}")]
    ShaderCreation(String),

    #[error("Failed to create graphics pipeline: {0}")]
    PipelineCreation(String),

    #[error("Failed to create framebuffer: {0}")]
    FramebufferCreation(String),

    #[error("Failed to create command pool: {0}")]
    CommandPoolCreation(String),

    #[error("Failed to allocate memory: {0}")]
    MemoryAllocation(String),

    #[error("Failed to create render pass: {0}")]
    RenderPassCreation(String),

    #[error("Failed to create synchronization primitive: {0}")]
    SyncCreation(String),

    #[error("Synchronization error: {0}")]
    SyncError(String),

    #[error("No suitable GPU found")]
    NoSuitableGpu,

    #[error("No suitable memory type found")]
    NoSuitableMemory,

    #[error("Vulkan validation error: {0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, VulkanError>;
