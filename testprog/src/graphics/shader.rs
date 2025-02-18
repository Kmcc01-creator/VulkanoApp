use std::sync::Arc;
use vulkano::device::Device;
use vulkano::shader::{ShaderModule, ShaderStages};

use crate::core::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

pub struct Shader {
    module: Arc<ShaderModule>,
}

impl Shader {
    #[allow(deprecated)]
    pub fn new(device: Arc<Device>, code: &[u32], ty: ShaderType) -> Result<Self, Error> {
        let _stage = match ty {
            ShaderType::Vertex => ShaderStages::VERTEX,
            ShaderType::Fragment => ShaderStages::FRAGMENT,
        };

        let bytes = bytemuck::cast_slice(code);
        let module = unsafe {
            ShaderModule::from_bytes(device, bytes).map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to create shader module: {}", e))
            })?
        };

        Ok(Self { module })
    }

    pub fn as_ref(&self) -> &Arc<ShaderModule> {
        &self.module
    }
}

// Helper macro for loading SPIR-V shaders
#[macro_export]
macro_rules! load_shader {
    ($device:expr, $path:expr, $ty:expr) => {{
        let code = std::fs::read($path).map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to read shader file {}: {}", $path, e))
        })?;

        if code.len() % 4 != 0 {
            return Err(Error::GraphicsInitialization(format!(
                "Shader file {} size is not a multiple of 4",
                $path
            )));
        }

        // Convert the bytes to u32 slice
        let words =
            unsafe { std::slice::from_raw_parts(code.as_ptr() as *const u32, code.len() / 4) };

        Shader::new($device, words, $ty)
    }};
}
