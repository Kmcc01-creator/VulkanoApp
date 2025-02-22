use ash::vk;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use ashengine::{
    context::Context,
    error::{Result, VulkanError},
    memory::{Buffer, MemoryAllocator},
    pipeline::Pipeline,
    swapchain::Swapchain,
    text::{vertex::TextVertex, FontAtlas},
};

// Keep all previous code...
// (TextRenderer implementation, FrameResources struct, helper functions, etc.)

fn create_shader_module(context: Arc<Context>, code: &[u8]) -> Result<vk::ShaderModule> {
    let create_info = vk::ShaderModuleCreateInfo::builder().code(bytemuck::cast_slice(code));

    unsafe {
        context
            .device()
            .create_shader_module(&create_info, None)
            .map_err(|e| VulkanError::ShaderCreation(e.to_string()))
    }
}

fn create_render_pass(context: Arc<Context>, format: vk::Format) -> Result<vk::RenderPass> {
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

    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(std::slice::from_ref(&color_attachment))
        .subpasses(std::slice::from_ref(&subpass));

    unsafe {
        context
            .device()
            .create_render_pass(&render_pass_info, None)
            .map_err(|e| VulkanError::RenderPassCreation(e.to_string()))
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Text Rendering Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)?;

    let context = Arc::new(Context::new()?);
    let surface = unsafe {
        let entry = ash::Entry::linked();
        ash_window::create_surface(
            &entry,
            context.instance(),
            window.raw_display_handle(),
            window.raw_window_handle(),
            None,
        )
        .map_err(|e| VulkanError::SurfaceCreation(e.to_string()))?
    };

    let mut renderer = TextRenderer::new(context.clone(), surface, 800, 600)?;

    // Load font and prepare text
    renderer.load_font("default", "assets/fonts/RobotoMono-Regular.ttf")?;
    renderer.update_text("Hello, World!", [-0.5, 0.0], "default", 32.0)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if let Err(e) = renderer.render() {
                    eprintln!("Render error: {}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    });
}
