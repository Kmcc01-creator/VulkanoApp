use std::sync::Arc;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo};
use vulkano::format::Format;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::pipeline::graphics::{
    color_blend::ColorBlendState,
    input_assembly::PrimitiveTopology,
    vertex_input::Vertex,
    viewport::{Viewport, ViewportState},
    GraphicsPipelineCreateInfo,
};
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineLayoutCreateInfo};
use vulkano::render_pass::{
    AttachmentDescription, LoadOp, RenderPass, RenderPassCreateInfo, StoreOp, SubpassDescription,
};
use vulkano::shader::{ShaderModule, ShaderStages};

use testprog::graphics::pipeline::PipelineCache;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
struct TestVertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(TestVertex, position);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create instance and select device
    let instance = Instance::new(InstanceCreateInfo::default())?;

    let physical = instance
        .enumerate_physical_devices()?
        .find(|p| p.properties().device_type == PhysicalDeviceType::DiscreteGpu)
        .unwrap_or_else(|| {
            instance
                .enumerate_physical_devices()
                .unwrap()
                .next()
                .expect("No devices available")
        });

    // Create device and queues
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("No graphics queue family available");

    let (device, mut queues) = Device::new(
        physical,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index: queue_family.index(),
                ..Default::default()
            }],
            ..Default::default()
        },
    )?;

    // Create render pass
    let render_pass = RenderPass::new(
        device.clone(),
        RenderPassCreateInfo {
            attachments: vec![AttachmentDescription {
                format: Format::R8G8B8A8_UNORM,
                samples: 1,
                load_op: LoadOp::Clear,
                store_op: StoreOp::Store,
                ..Default::default()
            }],
            subpasses: vec![SubpassDescription {
                color_attachments: vec![Some(0)],
                ..Default::default()
            }],
            ..Default::default()
        },
    )?;

    // Create pipeline layout
    let layout = PipelineLayout::new(device.clone(), PipelineLayoutCreateInfo::default())?;

    // Initialize pipeline cache
    let mut pipeline_cache = PipelineCache::new(device.clone());

    // Create viewport
    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [800.0, 600.0],
        depth_range: 0.0..1.0,
    };

    // Create pipeline info
    let mut create_info = GraphicsPipelineCreateInfo::vertex_input_state(Default::default())
        .input_assembly_state(Default::default())
        .viewport_state(ViewportState {
            viewports: smallvec::smallvec![viewport.clone()],
            scissors: smallvec::smallvec![viewport.rect()],
        })
        .color_blend_state(ColorBlendState::default());

    // Try to create pipeline (note: in real usage you'd need actual shader modules)
    let test_shaders = vec![(ShaderModule::new(device.clone(), &[]).unwrap(), "main")];

    // Create pipeline
    let pipeline = pipeline_cache.get_or_create(
        create_info.clone(),
        layout.clone(),
        render_pass.clone(),
        &test_shaders,
        Some(viewport.clone()),
    )?;

    // Test pipeline caching by creating the same pipeline again
    let cached_pipeline = pipeline_cache.get_or_create(
        create_info,
        layout,
        render_pass,
        &test_shaders,
        Some(viewport),
    )?;

    println!("Pipeline cache stats: {:?}", pipeline_cache.get_stats());

    Ok(())
}
