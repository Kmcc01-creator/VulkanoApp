use ash::vk;
use std::sync::Arc;

use ashengine::{
    config::{ConfigLoader, ConfigManager, UIConfig},
    text::{FontAtlas, TextElement, TextLayout, TextPicker},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize config system
    let config_manager = Arc::new(ConfigManager::new());
    let mut config_loader = ConfigLoader::new(Arc::clone(&config_manager))?;

    // Load UI configuration
    config_loader.load_config("examples/ui_config.ron")?;

    // Enable hot-reloading of configs
    config_loader.enable_hot_reload()?;

    // Get UI configuration
    let ui_config = config_manager
        .get::<UIConfig>("ui")
        .expect("UI config not found");
    let ui_config = ui_config.read().unwrap();

    // Create text elements with configured properties
    let text_elements = vec![
        TextElement {
            text: "Hello World".to_string(),
            position: [10.0, 10.0],
            color: ui_config.themes["default"].colors["text"],
            scale: ui_config.text.font_size / 32.0, // Normalize to SDF font size
            element_id: 1,
        },
        TextElement {
            text: "Click me!".to_string(),
            position: [10.0, 50.0],
            color: ui_config.themes["default"].colors["primary"],
            scale: ui_config.text.font_size / 32.0,
            element_id: 2,
        },
    ];

    // Initialize text rendering components
    let font_atlas = FontAtlas::new(
        device.clone(), // device would be your vulkan device
        512,            // atlas width
        512,            // atlas height
    )?;

    let mut text_layout = TextLayout::new();

    // Apply layout with configured padding
    let layout_config = &ui_config.layout;
    text_layout.layout_text(&text_elements, &font_atlas);

    // Setup text picker with configured properties
    let text_picker = TextPicker::new(device.clone())?;

    // Example of handling text selection
    let ray_origin = [100.0, 100.0]; // Mouse position
    let ray_direction = [1.0, 0.0]; // Ray direction

    text_picker.test_intersection(
        command_buffer, // Your command buffer
        bbox_buffer,    // Buffer containing text bounding boxes
        result_buffer,  // Buffer for storing intersection result
        descriptor_set, // Descriptor set for the compute shader
        ray_origin,
        ray_direction,
        text_elements.len() as u32,
    );

    // Example of updating configuration at runtime
    config_manager.update("ui", |config: &mut UIConfig| {
        config.text.font_size = 20.0;
        config.active_theme = "light".to_string();
    });

    Ok(())
}
