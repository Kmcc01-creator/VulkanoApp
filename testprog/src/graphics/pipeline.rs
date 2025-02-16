use std::sync::Arc;
use vulkano::device::Device;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::rasterization::{CullMode, FrontFace, RasterizationState};
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout};
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::shader::ShaderModule;

use super::vertex::Vertex2D;
use crate::core::Error;

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
                push_constant_ranges: vec![],
                set_layouts: vec![],
                ..Default::default()
            },
        )
        .map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to create pipeline layout: {}", e))
        })?;

        let stages = [
            (
                vertex_shader,
                vulkano::pipeline::GraphicsEntryPoint::new("main"),
            ),
            (
                fragment_shader,
                vulkano::pipeline::GraphicsEntryPoint::new("main"),
            ),
        ];

        let subpass = Subpass::from(render_pass, 0).ok_or_else(|| {
            Error::GraphicsInitialization("Failed to create subpass from render pass".into())
        })?;

        let pipeline = GraphicsPipeline::new(
            device,
            None,
            GraphicsPipelineCreateInfo {
                stages: &stages,
                vertex_input_state: Some(Vertex2D::per_vertex()),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into(),
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

        Ok(Self {
            pipeline: pipeline.into(),
        })
    }

    pub fn pipeline(&self) -> &Arc<GraphicsPipeline> {
        &self.pipeline
    }
}
