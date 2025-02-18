use smallvec::SmallVec;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::rasterization::{CullMode, FrontFace, RasterizationState};
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::shader::{ShaderModule, ShaderStages};

use crate::core::Error;
use crate::graphics::vertex::Vertex2D;

pub struct RenderPipeline {
    pipeline: Arc<GraphicsPipeline>,
}

impl RenderPipeline {
    pub fn new(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        vertex_shader: Arc<ShaderModule>,
        fragment_shader: Arc<ShaderModule>,
        viewport: Viewport,
    ) -> Result<Self, Error> {
        let pipeline_layout = PipelineLayout::new(
            device.clone(),
            PipelineLayoutCreateInfo {
                push_constant_ranges: vec![vulkano::pipeline::layout::PushConstantRange {
                    offset: 0,
                    size: 4,
                    stages: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                }],
                set_layouts: vec![],
                ..Default::default()
            },
        )
        .map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to create pipeline layout: {}", e))
        })?;

        let subpass = Subpass::from(render_pass.clone(), 0).ok_or_else(|| {
            Error::GraphicsInitialization("Failed to create subpass from render pass".into())
        })?;

        let vertex_input_state = Vertex2D::per_vertex()
            .definition(
                &vertex_shader
                    .entry_point("main")
                    .unwrap()
                    .info()
                    .input_interface,
            )
            .unwrap();

        let vertex_stage = vertex_shader.entry_point("main").ok_or_else(|| {
            Error::GraphicsInitialization("Vertex shader entry point not found".into())
        })?;

        let fragment_stage = fragment_shader.entry_point("main").ok_or_else(|| {
            Error::GraphicsInitialization("Fragment shader entry point not found".into())
        })?;

        let stages = {
            let mut stages: SmallVec<[_; 5]> = SmallVec::new();
            stages.push(PipelineShaderStageCreateInfo::new(vertex_stage));
            stages.push(PipelineShaderStageCreateInfo::new(fragment_stage));
            stages
        };

        let pipeline = GraphicsPipeline::new(
            device,
            None,
            GraphicsPipelineCreateInfo {
                stages,
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: vec![viewport].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState {
                    cull_mode: CullMode::Back,
                    front_face: FrontFace::Clockwise,
                    ..Default::default()
                }),
                multisample_state: None,
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(pipeline_layout)
            },
        )
        .map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to create graphics pipeline: {}", e))
        })?;

        Ok(Self { pipeline })
    }

    pub fn pipeline(&self) -> &Arc<GraphicsPipeline> {
        &self.pipeline
    }
}
