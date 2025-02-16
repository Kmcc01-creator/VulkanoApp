mod device;
mod pipeline;
mod renderer;
mod resource;
mod shader;
mod swapchain;
mod vertex;

pub use device::DeviceContext;
pub use pipeline::RenderPipeline;
pub use renderer::RenderContext;
pub use resource::{Material, MaterialBuilder, MaterialProperties, ShaderCache};
pub use shader::{ShaderModule, ShaderType};
pub use swapchain::SwapchainContext;
pub use vertex::{Mesh, Vertex2D, Vertex3D};

use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::{ImageLayout, ImageUsage, SampleCount};
use vulkano::render_pass::{
    AttachmentDescription, AttachmentReference, RenderPass, RenderPassCreateInfo,
    SubpassDescription,
};
use vulkano::sync::GpuFuture;

use crate::core::Error;
use std::sync::Arc;

pub fn create_render_pass(device: Arc<Device>, format: Format) -> Result<Arc<RenderPass>, Error> {
    let render_pass = RenderPass::new(
        device,
        RenderPassCreateInfo {
            flags: Default::default(),
            attachments: vec![AttachmentDescription {
                format: Some(format),
                samples: SampleCount::Sample1,
                load_op: vulkano::render_pass::LoadOp::Clear,
                store_op: vulkano::render_pass::StoreOp::Store,
                stencil_load_op: vulkano::render_pass::LoadOp::DontCare,
                stencil_store_op: vulkano::render_pass::StoreOp::DontCare,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::PresentSrc,
                ..Default::default()
            }],
            subpasses: vec![SubpassDescription {
                color_attachments: vec![Some(AttachmentReference {
                    attachment: 0,
                    layout: ImageLayout::ColorAttachmentOptimal,
                    ..Default::default()
                })],
                ..Default::default()
            }],
            dependencies: vec![],
            correlated_view_masks: vec![],
            _ne: Default::default(),
        },
    )
    .map_err(|e| Error::GraphicsInitialization(format!("Failed to create render pass: {}", e)))?;

    Ok(render_pass)
}
