use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use ashengine::{
    context::Context,
    error::{Result, VulkanError},
    renderer::Renderer,
    text::{FontAtlas, TextLayout, TextPicker},
};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() -> Result<()> {
    pretty_env_logger::init();

    // Create window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Text Engine Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .map_err(|e| VulkanError::WindowError(e.to_string()))?;

    // Initialize Vulkan
    let context = Context::new(Some(&window))?;
    let device = context.device();

    // Initialize text rendering components
    let font_atlas = FontAtlas::new(device.clone(), 512, 512)?;
    let mut text_layout = TextLayout::new();
    let text_picker = TextPicker::new(device.clone())?;

    // Create renderer
    let mut renderer = Renderer::new(context)?;

    // Main event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if let Err(e) = render_frame(&mut renderer) {
                    println!("Failed to render frame: {}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    })
}

fn render_frame(renderer: &mut Renderer) -> Result<()> {
    renderer.begin_frame()?;

    // Record commands here
    let command_buffer = renderer.current_command_buffer();

    // End frame
    renderer.end_frame()?;

    Ok(())
}
