use crate::error::{Result, VulkanError};
use ash::{vk, Device};
use std::sync::Arc;

pub struct RenderPass {
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    device: Arc<Device>,
}

impl RenderPass {
    pub fn new(
        device: Arc<Device>,
        format: vk::Format,
        image_views: &[vk::ImageView],
        extent: vk::Extent2D,
    ) -> Result<Self> {
        // Color attachment description
        let color_attachment = vk::AttachmentDescription::builder()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .build();

        // Subpass dependency to ensure proper image layout transitions
        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        let render_pass = unsafe {
            device
                .create_render_pass(&render_pass_info, None)
                .map_err(|e| VulkanError::RenderPassCreation(e.to_string()))?
        };

        // Create framebuffers
        let framebuffers = Self::create_framebuffers(&device, render_pass, image_views, extent)?;

        Ok(Self {
            render_pass,
            framebuffers,
            device,
        })
    }

    fn create_framebuffers(
        device: &Device,
        render_pass: vk::RenderPass,
        image_views: &[vk::ImageView],
        extent: vk::Extent2D,
    ) -> Result<Vec<vk::Framebuffer>> {
        image_views
            .iter()
            .map(|&image_view| {
                let attachments = [image_view];
                let framebuffer_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);

                unsafe {
                    device
                        .create_framebuffer(&framebuffer_info, None)
                        .map_err(|e| VulkanError::FramebufferCreation(e.to_string()))
                }
            })
            .collect()
    }

    pub fn begin_render_pass(
        &self,
        command_buffer: vk::CommandBuffer,
        framebuffer_index: usize,
        extent: vk::Extent2D,
        clear_color: [f32; 4],
    ) {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: clear_color,
            },
        }];

        let render_pass_begin = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[framebuffer_index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            })
            .clear_values(&clear_values);

        unsafe {
            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin,
                vk::SubpassContents::INLINE,
            );
        }
    }

    pub fn handle(&self) -> vk::RenderPass {
        self.render_pass
    }

    pub fn framebuffers(&self) -> &[vk::Framebuffer] {
        &self.framebuffers
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            for &framebuffer in &self.framebuffers {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            self.device.destroy_render_pass(self.render_pass, None);
        }
    }
}
