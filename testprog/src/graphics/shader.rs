use crate::core::Error;
use std::sync::Arc;
use vulkano::pipeline::shader::ShaderEntryPoint;
use vulkano::shader::ShaderModule as VulkanShaderModule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

pub struct ShaderModule {
    module: Arc<VulkanShaderModule>,
    entry_point: String,
}

impl ShaderModule {
    pub fn new(module: Arc<VulkanShaderModule>, entry_point: impl Into<String>) -> Self {
        Self {
            module,
            entry_point: entry_point.into(),
        }
    }

    pub fn entry_point(&self) -> Result<ShaderEntryPoint, Error> {
        self.module.entry_point(&self.entry_point).ok_or_else(|| {
            Error::GraphicsInitialization("Shader entry point not found".to_string())
        })
    }

    pub fn module(&self) -> &Arc<VulkanShaderModule> {
        &self.module
    }
}
