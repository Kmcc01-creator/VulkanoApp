use ash::vk;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextBlocksConfig {
    text_settings: TextSettings,
    theme: Theme,
    blocks: Vec<TextBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextSettings {
    default_font: String,
    font_size: f32,
    line_height: f32,
    letter_spacing: f32,
    sdf_settings: SDFSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SDFSettings {
    smoothing: f32,
    thickness: f32,
    padding: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Theme {
    colors: HashMap<String, [f32; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextBlock {
    id: String,
    content: String,
    position: [f32; 2],
    color: String,
    scale: f32,
    selectable: bool,
}

impl Config for TextBlocksConfig {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn module_name(&self) -> &str {
        "text_blocks"
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create window and event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Text Blocks Example")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)?;

    // Initialize vulkan context with window
    let context = Context::new(Some(&window))?;
    let device = context.device();

    // Initialize configuration
    let config_manager = Arc::new(ConfigManager::new());
    let mut config_loader = ConfigLoader::new(config_manager.clone())?;

    // Load text blocks configuration
    config_loader.load_config("examples/text_blocks.ron")?;
    let config = config_manager
        .get::<TextBlocksConfig>("text_blocks")?
        .read()
        .unwrap();

    // Initialize text rendering components
    let font_atlas = FontAtlas::new(device.clone(), 512, 512)?;
    let mut text_layout = TextLayout::new();
    let text_picker = TextPicker::new(device.clone())?;

    // Create text elements from configuration
    let text_elements: Vec<TextElement> = config
        .blocks
        .iter()
        .enumerate()
        .map(|(idx, block)| TextElement {
            text: block.content.clone(),
            position: block.position,
            color: config.theme.colors[&block.color],
            scale: block.scale * (config.text_settings.font_size / 32.0),
            element_id: if block.selectable { idx as u32 + 1 } else { 0 },
        })
        .collect();

    // Create descriptor sets and pipeline
    let descriptor_pool = create_descriptor_pool(&device)?;
    let descriptor_set_layout = create_descriptor_set_layout(&device)?;
    let pipeline_layout = create_pipeline_layout(&device, &[descriptor_set_layout])?;
    let descriptor_sets =
        allocate_descriptor_sets(&device, descriptor_pool, &[descriptor_set_layout])?;

    // Create renderer
    let mut renderer = Renderer::new(context.clone())?;

    // Create buffers for text data
    let vertex_buffer = create_vertex_buffer(&device, &text_layout.vertices())?;
    let index_buffer = create_index_buffer(&device, &text_layout.indices())?;
    let bbox_buffer = create_storage_buffer(&device, &text_layout.bounding_boxes())?;
    let result_buffer = create_storage_buffer(&device, &[0u32, 0f32])?; // For picking results

    // Layout text elements
    text_layout.layout_text(&text_elements, &font_atlas);

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
                // Convert cursor position to normalized device coordinates
                let size = window.inner_size();
                let x = (position.x as f32 / size.width as f32) * 2.0 - 1.0;
                let y = (position.y as f32 / size.height as f32) * 2.0 - 1.0;

                // Test for text intersection
                text_picker.test_intersection(
                    renderer.current_command_buffer(),
                    bbox_buffer,
                    result_buffer,
                    descriptor_sets[0],
                    [x, y],
                    [0.0, 1.0], // Ray direction for 2D picking
                    text_elements.len() as u32,
                );
            }

            Event::RedrawRequested(_) => {
                renderer.begin_frame().unwrap();

                let command_buffer = renderer.current_command_buffer();
                let extent = renderer.swapchain_extent();

                // Set viewport and scissor
                renderer.cmd_set_viewport(command_buffer, extent);
                renderer.cmd_set_scissor(command_buffer, extent);

                // Bind pipeline and vertex buffers
                unsafe {
                    device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
                    device.cmd_bind_index_buffer(
                        command_buffer,
                        index_buffer,
                        0,
                        vk::IndexType::UINT32,
                    );
                }

                // Draw text
                unsafe {
                    device.cmd_draw_indexed(command_buffer, text_layout.index_count(), 1, 0, 0, 0);
                }

                renderer.end_frame().unwrap();
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            _ => (),
        }
    });
}
