use std::sync::Arc;
use vulkano::device::Device;
use vulkano::pipeline::{
    graphics::{
        input_assembly::InputAssemblyState, vertex_input::Vertex as VulkanoVertex,
        vertex_input::VertexInputState, viewport::Viewport, viewport::ViewportState,
        GraphicsPipelineCreateInfo,
    },
    GraphicsPipeline,
};
use vulkano::render_pass::Subpass;
use vulkano::shader::ShaderModule;

use super::shader::{ShaderModule as CustomShaderModule, ShaderType};
use super::vertex::Vertex;
use crate::core::error::Error;

pub struct PipelineBuilder {
    device: Arc<Device>,
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    viewport: Viewport,
}

impl PipelineBuilder {
    pub fn new(
        device: Arc<Device>,
        vertex_shader: Arc<ShaderModule>,
        fragment_shader: Arc<ShaderModule>,
    ) -> Self {
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [0.0, 0.0],
            depth_range: 0.0..=1.0,
        };

        Self {
            device,
            vertex_shader,
            fragment_shader,
            viewport,
        }
    }

    pub fn with_viewport(mut self, viewport: Viewport) -> Self {
        self.viewport = viewport;
        self
    }

    pub fn build(self, subpass: Subpass) -> Result<Arc<GraphicsPipeline>, Error> {
        let vertex_shader_entry =
            self.vertex_shader
                .module()
                .entry_point("main")
                .ok_or_else(|| {
                    Error::GraphicsInitialization("Vertex shader entry point not found".to_string())
                })?;

        let fragment_shader_entry = self
            .fragment_shader
            .module()
            .entry_point("main")
            .ok_or_else(|| {
                Error::GraphicsInitialization("Fragment shader entry point not found".to_string())
            })?;

        let pipeline = GraphicsPipeline::new(
            self.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                vertex_input_state: Some(Vertex::per_vertex()),
                input_assembly_state: Some(InputAssemblyState::default()),
                vertex_shader_state: vertex_shader_entry,
                fragment_shader_state: Some(fragment_shader_entry),
                viewport_state: Some(ViewportState {
                    viewports: vec![self.viewport].into(),
                    ..Default::default()
                }),
                subpass: Some(subpass),
                ..Default::default()
            },
        )
        .map_err(|e| Error::GraphicsInitialization(e.to_string()))?;

        Ok(pipeline)
    }
}

pub struct RenderPipeline {
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
}

impl RenderPipeline {
    pub fn new(pipeline: Arc<GraphicsPipeline>, viewport: Viewport) -> Self {
        Self { pipeline, viewport }
    }

    pub fn raw_pipeline(&self) -> &Arc<GraphicsPipeline> {
        &self.pipeline
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }
}
