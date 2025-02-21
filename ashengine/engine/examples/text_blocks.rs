use ash::vk;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use ashengine::{
    config::{Config, ConfigLoader, ConfigManager},
    context::Context,
    error::{Result as VkResult, VulkanError},
    helpers::{
        allocate_descriptor_sets, create_descriptor_pool, create_descriptor_set_layout,
        create_index_buffer, create_pipeline_layout, create_storage_buffer, create_vertex_buffer,
    },
    text::{FontAtlas, TextElement, TextLayout, TextPicker},
    RenderPass, Renderer, Swapchain,
};

use std::error::Error;

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

    // Initialize Vulkan context and renderer
    let context = Context::new(Some(&window))?;
    let device = context.device();

    // Create renderer
    let mut renderer = Renderer::new(
        device.clone(),
        context.graphics_queue(),
        context.queue_family_index(),
        context.physical_device(),
        context.instance(),
        context.surface_loader(),
        context.surface(),
    )?;

    // Initialize configuration
    let config_manager = Arc::new(ConfigManager::new());
    let mut config_loader = ConfigLoader::new(config_manager.clone())?;

    // Load text blocks configuration
    config_loader.load_config("examples/text_blocks.ron")?;
    let config = config_manager
        .get::<TextBlocksConfig>("text_blocks")
        .ok_or_else(|| Box::new(VulkanError::General("Failed to get config".into())))?
        .read()
        .map_err(|e| Box::new(VulkanError::General(e.to_string())))?;

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

    // Layout text elements
    text_layout.layout_text(&text_elements, &font_atlas);

    // Create swapchain
    let swapchain = Swapchain::new(
        context.physical_device(),
        device.clone(),
        context.instance(),
        context.surface_loader(),
        context.surface(),
        [800, 600],
    )?;

    // Create render pass
    let render_pass = RenderPass::new(
        device.clone(),
        swapchain.surface_format().format,
        swapchain.image_views(),
        swapchain.extent(),
    )?;

    // Create descriptor sets and pipeline
    let pool_sizes = [vk::DescriptorPoolSize {
        ty: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 2,
    }];
    let descriptor_pool = create_descriptor_pool(&device, 1, &pool_sizes)?;

    let bindings = [
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .build(),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .build(),
    ];
    let descriptor_set_layout = create_descriptor_set_layout(&device, &bindings)?;
    let pipeline_layout = create_pipeline_layout(&device, &[descriptor_set_layout])?;
    let descriptor_sets =
        allocate_descriptor_sets(&device, descriptor_pool, &[descriptor_set_layout])?;

    // Create buffers for text data
    let vertex_data = text_layout.vertices();
    let index_data = text_layout.indices();
    let bbox_data = text_layout.bounding_boxes();

    let (vertex_buffer, vertex_memory) = create_vertex_buffer(&device, &vertex_data)?;
    let (index_buffer, index_memory) = create_index_buffer(&device, &index_data)?;
    let (bbox_buffer, bbox_memory) = create_storage_buffer(&device, &bbox_data)?;
    let (result_buffer, result_memory) = create_storage_buffer(&device, &[0u32, 0])?;
    // Initialize renderer with swapchain and shaders
    renderer.initialize_swapchain(swapchain, render_pass, &[], &[])?; // TODO: Add shader loading
    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        min_depth: 0.0,
        max_depth: 1.0,
    };

    let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D {
            width: 800,
            height: 600,
        },
    };

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
                if let Err(e) = renderer.begin_frame() {
                    eprintln!("Failed to begin frame: {}", e);
                    return;
                }

                let command_buffer = renderer.current_command_buffer();

                unsafe {
                    // Set viewport and scissor
                    device.cmd_set_viewport(command_buffer, &[viewport]);
                    device.cmd_set_scissor(command_buffer, &[scissor]);

                    // Bind vertex and index buffers
                    device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer.0], &[0]);
                    device.cmd_bind_index_buffer(
                        command_buffer,
                        index_buffer.0,
                        0,
                        vk::IndexType::UINT32,
                    );

                    // Draw text
                    device.cmd_draw_indexed(command_buffer, text_layout.index_count(), 1, 0, 0, 0);
                }

                if let Err(e) = renderer.end_frame() {
                    eprintln!("Failed to end frame: {}", e);
                }
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            _ => (),
        }
    });
}
