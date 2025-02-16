use std::sync::Arc;
use vulkano::device::Device;
use vulkano::shader::{ShaderCreateInfo, ShaderModule as VkShaderModule};

use crate::core::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

pub struct ShaderModule {
    module: Arc<VkShaderModule>,
}

impl ShaderModule {
    pub fn new(device: Arc<Device>, code: &[u32], ty: ShaderType) -> Result<Self, Error> {
        let stage = match ty {
            ShaderType::Vertex => vulkano::shader::ShaderStage::Vertex,
            ShaderType::Fragment => vulkano::shader::ShaderStage::Fragment,
        };

        let module = unsafe {
            VkShaderModule::new(
                device,
                ShaderCreateInfo {
                    code: code.into(),
                    ..Default::default()
                },
            )
        }
        .map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to create shader module: {}", e))
        })?;

        Ok(Self {
            module: Arc::new(module),
        })
    }

    pub fn as_ref(&self) -> Arc<VkShaderModule> {
        self.module.clone()
    }
}

// Helper macro for loading SPIR-V shaders
#[macro_export]
macro_rules! load_shader {
    ($device:expr, $path:expr, $ty:expr) => {{
        let code = std::fs::read($path).map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to read shader file {}: {}", $path, e))
        })?;
        let code = Vec::from(code);
        let code = if code.len() % 4 == 0 {
            unsafe {
                Vec::from_raw_parts(code.as_ptr() as *mut u32, code.len() / 4, code.capacity())
            }
        } else {
            return Err(Error::GraphicsInitialization(format!(
                "Shader file {} size is not a multiple of 4",
                $path
            )));
        };
        ShaderModule::new($device, &code, $ty)
    }};
}
