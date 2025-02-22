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
    Result, VulkanError,
};
use log::info;
// use serde::Deserialize; // Temporarily disabling serde deserialization
use std::collections::HashMap;
use std::fs::read_to_string;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// #[derive(Deserialize, Debug, Clone)] // Temporarily disabling serde deserialization
#[derive(Debug, Clone)]
struct TextSettings {
    default_font: String,
    font_size: f32,
    line_height: f32,
    letter_spacing: f32,
    sdf_settings: SdfSettings,
}

// #[derive(Deserialize, Debug, Clone)] // Temporarily disabling serde deserialization
#[derive(Debug, Clone)]
struct SdfSettings {
    smoothing: f32,
    thickness: f32,
    padding: f32,
}

// #[derive(Deserialize, Debug, Clone)] // Temporarily disabling serde deserialization
#[derive(Debug, Clone)]
struct Theme {
    colors: HashMap<String, [f32; 4]>,
}

// #[derive(Deserialize, Debug, Clone)] // Temporarily disabling serde deserialization
#[derive(Debug, Clone)]
struct TextBlock {
    id: String,
    content: String,
    position: [f32; 2],
    color: String,
    scale: f32,
    selectable: bool,
}

// #[derive(Deserialize, Debug, Clone)] // Temporarily disabling serde deserialization
#[derive(Debug, Clone)]
struct Config {
    text_settings: TextSettings,
    theme: Theme,
    blocks: Vec<TextBlock>,
}

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

    // Load text configuration
    info!("Loading text configuration");
    let config_str = read_to_string("engine/examples/text_blocks.toml")
        .map_err(|e| VulkanError::General(format!("Failed to read config file: {}", e)))?;
    // let config: Config = toml::from_str(&config_str)?; // Temporarily disabling toml deserialization

    // Manual Config Construction (Temporary Workaround)
    let config = {
        let mut blocks: Vec<TextBlock> = Vec::new();
        let mut text_settings = TextSettings {
            default_font: "Arial".to_string(),
            font_size: 16.0,
            line_height: 1.2,
            letter_spacing: 0.0,
            sdf_settings: SdfSettings {
                smoothing: 0.25,
                thickness: 0.5,
                padding: 4.0,
            },
        };
        let mut theme = Theme {
            colors: HashMap::new(),
        };

        // Very basic parsing -  assumes structure and doesn't handle errors
        for line in config_str.lines() {
            if line.starts_with("default_font") {
                text_settings.default_font = line
                    .split('=')
                    .nth(1)
                    .unwrap_or("Arial")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            } else if line.starts_with("font_size") {
                text_settings.font_size = line
                    .split('=')
                    .nth(1)
                    .unwrap_or("16.0")
                    .trim()
                    .parse()
                    .unwrap_or(16.0);
            } else if line.starts_with("[[blocks]]") {
                let mut block = TextBlock {
                    id: "".to_string(),
                    content: "".to_string(),
                    position: [0.0, 0.0],
                    color: "".to_string(),
                    scale: 1.0,
                    selectable: false,
                };
                blocks.push(block);
            } else if let Some(last_block) = blocks.last_mut() {
                if line.starts_with("id") {
                    last_block.id = line
                        .split('=')
                        .nth(1)
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                } else if line.starts_with("content") {
                    last_block.content = line
                        .split('=')
                        .nth(1)
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                } else if line.starts_with("position") {
                    let pos_str = line.split('=').nth(1).unwrap_or("[0.0, 0.0]").trim();
                    let pos_values: Vec<f32> = pos_str
                        .trim_matches(|c| c == '[' || c == ']')
                        .split(',')
                        .map(|s| s.trim().parse().unwrap_or(0.0))
                        .collect();
                    if pos_values.len() == 2 {
                        last_block.position = [pos_values[0], pos_values[1]];
                    }
                } else if line.starts_with("color") {
                    last_block.color = line
                        .split('=')
                        .nth(1)
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                } else if line.starts_with("scale") {
                    last_block.scale = line
                        .split('=')
                        .nth(1)
                        .unwrap_or("1.0")
                        .trim()
                        .parse()
                        .unwrap_or(1.0);
                } else if line.starts_with("selectable") {
                    last_block.selectable = line
                        .split('=')
                        .nth(1)
                        .unwrap_or("false")
                        .trim()
                        .parse()
                        .unwrap_or(false);
                }
            } else if line.starts_with("primary")
                || line.starts_with("secondary")
                || line.starts_with("highlight")
            {
                let color_name = line.split('=').next().unwrap_or("").trim();
                let color_str = line
                    .split('=')
                    .nth(1)
                    .unwrap_or("[1.0, 1.0, 1.0, 1.0]")
                    .trim();
                let color_values: Vec<f32> = color_str
                    .trim_matches(|c| c == '[' || c == ']')
                    .split(',')
                    .map(|s| s.trim().parse().unwrap_or(1.0))
                    .collect();
                if color_values.len() == 4 {
                    theme.colors.insert(
                        color_name.to_string(),
                        [
                            color_values[0],
                            color_values[1],
                            color_values[2],
                            color_values[3],
                        ],
                    );
                }
            }
        }

        Config {
            text_settings,
            theme,
            blocks,
        }
    };

    // Initialize text rendering components
    info!("Initializing text rendering components");
    let mut font_atlas = FontAtlas::new(context.clone(), 128, 128)?;

    // Load and generate glyphs
    info!("Loading font and generating glyphs");
    font_atlas.load_font(
        &config.text_settings.default_font,
        "engine/examples/fonts/NotoSans-Regular.ttf",
    )?;

    // Generate glyphs for the title
    for c in config.blocks[0].content.chars() {
        font_atlas.generate_glyph(
            c,
            &config.text_settings.default_font,
            config.text_settings.font_size * config.blocks[0].scale,
        )?;
    }

    // Generate glyphs for the subtitle
    for c in config.blocks[1].content.chars() {
        font_atlas.generate_glyph(
            c,
            &config.text_settings.default_font,
            config.text_settings.font_size * config.blocks[1].scale,
        )?;
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

    // Create text elements
    info!("Creating text elements");
    let mut text_elements: Vec<TextElement> = Vec::new();
    for block in &config.blocks {
        let color = config
            .theme
            .colors
            .get(&block.color)
            .unwrap_or(&[1.0, 1.0, 1.0, 1.0]);
        text_elements.push(TextElement {
            text: block.content.clone(),
            position: block.position,
            color: *color,
            scale: block.scale, // Use scale directly from config
            element_id: block.id.parse::<u32>().unwrap_or(0),
        });
    }

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
