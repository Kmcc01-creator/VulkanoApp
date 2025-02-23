use ash::vk;
use shaderc;
use std::sync::Arc;

pub struct ShaderModule {
    device: Arc<ash::Device>,
    module: vk::ShaderModule,
}

impl ShaderModule {
    pub fn new(
        device: Arc<ash::Device>,
        spirv_code: &[u32],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(spirv_code)
            .build();

        let module = unsafe { device.create_shader_module(&create_info, None)? };

        Ok(Self { device, module })
    }

    pub fn get_module(&self) -> vk::ShaderModule {
        self.module
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.module, None);
        }
    }
}

pub fn compile_shader(
    source: &str,
    shader_kind: shaderc::ShaderKind,
    entry_point: &str,
) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let compiler = shaderc::Compiler::new().ok_or("Failed to create shader compiler")?;

    let binary_result = compiler.compile_into_spirv(
        source,
        shader_kind,
        "shader.comp", // Arbitrary filename for error messages
        entry_point,
        None, // No compilation options
    )?;

    Ok(binary_result.as_binary().to_vec())
}
