use ash::vk;
use ashengine::helpers::{create_index_buffer, create_storage_buffer, create_vertex_buffer};
use ashengine::renderer::Renderer;
use ashengine::{
    config::{ConfigLoader, ConfigManager},
    context::Context,
    text::{
        pixel_to_ndc, FontAtlas, TextAlignment, TextConfig, TextElement, TextLayout, TextPicker,
    },
    Result,
};
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<()> {
    // Initialize window and event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Text Configuration Example")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)?;

    // Initialize Vulkan context
    let context = Arc::new(Context::new(Some(&window))?);
    let device = context.device();

    // Initialize configuration system
    let config_manager = Arc::new(ConfigManager::new());
    let mut config_loader = ConfigLoader::new(config_manager.clone())?;

    // Load text configuration
    config_loader.load_config("examples/text_blocks.ron")?;

    // Initialize text rendering components
    let font_atlas = FontAtlas::new(context.clone(), 512, 512)?;
    let mut text_layout = TextLayout::new();
    let text_picker = TextPicker::new(device.clone())?;

    // Create text configuration
    let text_config = TextConfig {
        font_size: 24.0,
        line_height: 1.5,
        letter_spacing: 0.1,
        alignment: TextAlignment::Left,
        color: [1.0, 1.0, 1.0, 1.0],
    };

    // Create text elements
    let mut text_elements = vec![
        TextElement {
            text: "Welcome to Text Configuration Demo".to_string(),
            position: pixel_to_ndc(50.0, 50.0, 800.0, 600.0),
            color: [1.0, 1.0, 1.0, 1.0],
            scale: text_config.font_size / 32.0,
            element_id: 1,
        },
        TextElement {
            text: "Click any text to select it".to_string(),
            position: pixel_to_ndc(50.0, 100.0, 800.0, 600.0),
            color: [0.2, 0.6, 1.0, 1.0],
            scale: text_config.font_size / 32.0,
            element_id: 2,
        },
    ];

    // Layout text elements
    text_layout.layout_text(&text_elements, &font_atlas);

    // Create necessary buffers
    let (vertex_buffer, vertex_memory) = create_vertex_buffer(&device, text_layout.vertices())?;
    let (index_buffer, index_memory) = create_index_buffer(&device, text_layout.indices())?;
    let (bbox_buffer, bbox_memory) = create_storage_buffer(&device, text_layout.bounding_boxes())?;

    // Create renderer
    let mut renderer = Renderer::new(
        device.clone(),
        context.graphics_queue(),
        context.queue_family_index(),
        context.physical_device(),
        context.instance().clone(),
        context.surface_loader().clone(),
        context.surface(),
    )?;

    // Main event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let size = window.inner_size();
                let x = position.x as f32 / size.width as f32;
                let y = position.y as f32 / size.height as f32;

                // Test for intersection
                test_intersection(&text_picker, &renderer, bbox_buffer, [x, y]);
            }

            Event::RedrawRequested(_) => {
                if let Err(e) =
                    render_frame(&mut renderer, &text_layout, vertex_buffer, index_buffer)
                {
                    eprintln!("Render error: {}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            _ => (),
        }
    });
}

fn test_intersection(
    text_picker: &TextPicker,
    renderer: &Renderer,
    bbox_buffer: vk::Buffer,
    cursor_pos: [f32; 2],
) {
    let (result_buffer, result_memory) =
        create_storage_buffer(renderer.device(), &[0u32, 0u32]).unwrap();

    text_picker.test_intersection(
        renderer.current_command_buffer(),
        bbox_buffer,
        result_buffer,
        text_picker.descriptor_set(),
        cursor_pos,
        [0.0, 1.0],
        2,
    );
}

fn render_frame(
    renderer: &mut Renderer,
    text_layout: &TextLayout,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
) -> Result<()> {
    renderer.begin_frame()?;

    // Record draw commands
    let command_buffer = renderer.current_command_buffer();

    unsafe {
        renderer
            .device()
            .cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);

        renderer.device().cmd_bind_index_buffer(
            command_buffer,
            index_buffer,
            0,
            vk::IndexType::UINT32,
        );

        renderer
            .device()
            .cmd_draw_indexed(command_buffer, text_layout.index_count(), 1, 0, 0, 0);
    }

    renderer.end_frame()?;
    Ok(())
}
