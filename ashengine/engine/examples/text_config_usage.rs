use ash::vk;
use ashengine::helpers::{create_index_buffer, create_storage_buffer, create_vertex_buffer};
use ashengine::renderer::Renderer;
use ashengine::{
    config::{ConfigLoader, ConfigManager},
    context::Context,
    render_pass::RenderPass,
    shader::ShaderSet,
    swapchain::Swapchain,
    text::{
        atlas::{FontAtlas, GlyphMetrics},
        layout::{Rect, TextElement},
        TextAlignment, TextConfig, TextLayout, TextPicker,
    },
    Result,
};
use log::info;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting Text Configuration Example");

    // Initialize window and event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Text Configuration Example")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)?;

    // Initialize Vulkan context
    info!("Initializing Vulkan context");
    let context = Arc::new(Context::new(Some(&window))?);
    let device = context.device();

    // Initialize configuration system
    info!("Setting up configuration system");
    let config_manager = Arc::new(ConfigManager::new());
    let mut config_loader = ConfigLoader::new(config_manager.clone())?;

    // Load text configuration
    info!("Loading text configuration");
    config_loader.load_config("examples/text_blocks.ron")?;

    // Initialize text rendering components
    info!("Initializing text rendering components");
    let mut font_atlas = FontAtlas::new(context.clone(), 512, 512)?;

    // Add some basic glyphs for testing
    info!("Creating test glyphs");
    for (i, c) in "Hello World".chars().enumerate() {
        let x = (i as f32 / 11.0) * 0.1; // Spread characters across 0.0-0.1 UV space
        font_atlas.add_glyph(
            c,
            Rect {
                x,
                y: 0.0,
                width: 0.08, // Character width in UV space
                height: 0.1, // Character height in UV space
            },
            GlyphMetrics {
                advance: 32.0,
                bearing: [0.0, 24.0],
                size: [24.0, 24.0],
            },
        );
    }

    let mut text_layout = TextLayout::new();
    let text_picker = TextPicker::new(device.clone())?;

    // Create shader set
    info!("Creating shader set");
    let shader_set = ShaderSet::new(
        device.clone(),
        "shaders/text.vert.spv",
        "shaders/text.frag.spv",
    )?;

    // Initialize swapchain
    info!("Initializing swapchain");
    let swapchain = Swapchain::new(
        context.physical_device(),
        device.clone(),
        context.instance().clone(),
        context.surface_loader().clone(),
        context.surface(),
        [800, 600],
    )?;

    // Create render pass
    info!("Creating render pass");
    let render_pass = RenderPass::new(
        device.clone(),
        swapchain.surface_format().format,
        swapchain.image_views(),
        swapchain.extent(),
    )?;

    // Create renderer with font atlas descriptor set layout
    info!("Creating renderer");
    let descriptor_set_layouts = [font_atlas.descriptor_set_layout()];
    let mut renderer = Renderer::new(
        device.clone(),
        context.graphics_queue(),
        context.queue_family_index(),
        context.physical_device(),
        context.instance().clone(),
        context.surface_loader().clone(),
        context.surface(),
        shader_set,
        &descriptor_set_layouts,
    )?;

    // Initialize renderer with swapchain and render pass
    info!("Initializing renderer with swapchain and render pass");
    renderer.initialize_swapchain(swapchain, render_pass)?;

    // Create text configuration
    info!("Setting up text configuration");
    let text_config = TextConfig {
        font_size: 24.0,
        line_height: 1.5,
        letter_spacing: 0.1,
        alignment: TextAlignment::Left,
        color: [1.0, 1.0, 1.0, 1.0],
    };

    // Create text elements
    info!("Creating text elements");
    let text_elements = vec![TextElement {
        text: "Hello".to_string(),
        position: [-0.5, 0.0], // Center of screen
        color: [1.0, 1.0, 1.0, 1.0],
        scale: text_config.font_size / 32.0,
        element_id: 1,
    }];

    // Layout text elements
    info!("Laying out text elements");
    text_layout.layout_text(&text_elements, &font_atlas);

    // Create necessary buffers
    info!("Creating vertex and index buffers");
    let (vertex_buffer, _vertex_memory) = create_vertex_buffer(&device, text_layout.vertices())?;
    let (index_buffer, _index_memory) = create_index_buffer(&device, text_layout.indices())?;
    let (bbox_buffer, _bbox_memory) = create_storage_buffer(&device, text_layout.bounding_boxes())?;

    info!("Setup complete, entering main event loop");

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
                if let Err(e) = render_frame(
                    &mut renderer,
                    &text_layout,
                    vertex_buffer,
                    index_buffer,
                    font_atlas.descriptor_set(),
                ) {
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
    let (result_buffer, _result_memory) =
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
    font_descriptor_set: vk::DescriptorSet,
) -> Result<()> {
    renderer.begin_frame()?;

    // Record draw commands
    let command_buffer = renderer.current_command_buffer();

    unsafe {
        // Bind font descriptor set
        renderer.device().cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            renderer.pipeline_layout(),
            0,
            &[font_descriptor_set],
            &[],
        );

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
