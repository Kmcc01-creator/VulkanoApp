use crate::error::{Result, VulkanError};
use ash::{vk, Device};
use std::sync::Arc;

pub struct Pipeline {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    device: Arc<Device>,
}

impl Pipeline {
    pub fn new(
        device: Arc<Device>,
        render_pass: vk::RenderPass,
        shader_stages: &[vk::PipelineShaderStageCreateInfo],
        extent: vk::Extent2D,
    ) -> Result<Self> {
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder();

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::TRUE,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::from_raw(
                vk::ColorComponentFlags::R.as_raw()
                    | vk::ColorComponentFlags::G.as_raw()
                    | vk::ColorComponentFlags::B.as_raw()
                    | vk::ColorComponentFlags::A.as_raw(),
            ),
        };

        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        // Create pipeline layout
        let layout_info = vk::PipelineLayoutCreateInfo::builder();

        let layout = unsafe {
            device
                .create_pipeline_layout(&layout_info, None)
                .map_err(|e| VulkanError::PipelineCreation(e.to_string()))?
        };

        let create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&create_info),
                    None,
                )
                .map_err(|e| VulkanError::PipelineCreation(e.1.to_string()))?[0]
        };

        Ok(Self {
            pipeline,
            layout,
            device,
        })
    }

    pub fn bind(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
        }
    }

    pub fn handle(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
