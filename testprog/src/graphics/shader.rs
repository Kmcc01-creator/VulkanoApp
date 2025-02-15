use crate::core::Error;
use std::sync::Arc;
use vulkano::shader::{ShaderModule as VulkanShaderModule, ShaderStages};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

impl ShaderType {
    pub fn to_shader_stage(&self) -> ShaderStages {
        match self {
            ShaderType::Vertex => ShaderStages::VERTEX,
            ShaderType::Fragment => ShaderStages::FRAGMENT,
            ShaderType::Compute => ShaderStages::COMPUTE,
        }
    }
}

pub struct ShaderModule {
    module: Arc<VulkanShaderModule>,
    entry_point: String,
    shader_type: ShaderType,
}

impl ShaderModule {
    pub fn new(
        module: Arc<VulkanShaderModule>,
        entry_point: impl Into<String>,
        shader_type: ShaderType,
    ) -> Self {
        Self {
            module,
            entry_point: entry_point.into(),
            shader_type,
        }
    }

    pub fn entry_point_info(&self) -> Result<EntryPointInfo, Error> {
        self.module.entry_point(&self.entry_point).ok_or_else(|| {
            Error::GraphicsInitialization("Shader entry point not found".to_string())
        })
    }

    pub fn module(&self) -> &Arc<VulkanShaderModule> {
        &self.module
    }

    pub fn shader_stage(&self) -> ShaderStages {
        self.shader_type.to_shader_stage()
    }
}

pub struct EntryPointInfo {
    pub name: String,
    pub stage: ShaderStages,
}
