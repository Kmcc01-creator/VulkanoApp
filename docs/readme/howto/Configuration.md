# Working with Configurations

This guide provides a practical introduction to using AshEngine's configuration system.

## Quick Start

### 1. Basic Setup

```rust
use ashengine::config::{ConfigLoader, ConfigManager, UIConfig};
use std::sync::Arc;

// Create config manager
let config_manager = Arc::new(ConfigManager::new());
let mut config_loader = ConfigLoader::new(config_manager.clone())?;

// Load a configuration file
config_loader.load_config("configs/ui_config.ron")?;
```

### 2. Creating a Configuration File

Create a new file `ui_config.ron` with basic settings:

```ron
(
    text: (
        default_font: "Arial",
        font_size: 16.0,
        line_height: 1.2,
    ),
    layout: (
        padding: (
            top: 5.0,
            right: 5.0,
            bottom: 5.0,
            left: 5.0,
        ),
    ),
    themes: {
        "default": (
            colors: {
                "text": [1.0, 1.0, 1.0, 1.0],
                "background": [0.1, 0.1, 0.1, 1.0],
            },
            fonts: {
                "regular": "Arial-Regular",
            },
        ),
    },
    active_theme: "default",
)
```

### 3. Accessing Configurations

```rust
// Get UI configuration
let ui_config = config_manager.get::<UIConfig>("ui")?;
let config = ui_config.read().unwrap();

// Access settings
let font_size = config.text.font_size;
let text_color = config.themes["default"].colors["text"];
```

### 4. Updating Configurations

```rust
config_manager.update("ui", |config: &mut UIConfig| {
    config.text.font_size = 20.0;
    config.active_theme = "light".to_string();
});
```

### 5. Enable Hot-Reloading

```rust
// Enable hot-reloading for all configurations
config_loader.enable_hot_reload()?;
```

## Common Tasks

### Creating Custom Configurations

1. Define your configuration structure:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub difficulty: String,
    pub max_players: u32,
    pub save_interval: f32,
}
```

2. Implement the Config trait:

```rust
impl Config for GameConfig {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    fn module_name(&self) -> &str { "game" }
}
```

3. Create a configuration file (`game_config.ron`):

```ron
(
    difficulty: "normal",
    max_players: 4,
    save_interval: 300.0,
)
```

4. Load and use the configuration:

```rust
config_loader.load_config("configs/game_config.ron")?;
let game_config = config_manager.get::<GameConfig>("game")?;
```

### Working with Themes

```rust
// Switch themes
config_manager.update("ui", |config: &mut UIConfig| {
    config.active_theme = "dark".to_string();
});

// Add new theme
config_manager.update("ui", |config: &mut UIConfig| {
    config.themes.insert("custom".to_string(), Theme {
        colors: {
            let mut colors = HashMap::new();
            colors.insert("text".to_string(), [0.8, 0.8, 0.8, 1.0]);
            colors
        },
        fonts: HashMap::new(),
    });
});
```

## Best Practices

1. **Organization**

   - Keep configuration files in a dedicated `configs` directory
   - Use meaningful file names (e.g., `ui_config.ron`, `graphics_config.ron`)
   - Group related settings in the same configuration file

2. **Error Handling**

   - Always handle configuration loading errors
   - Provide sensible defaults for all settings
   - Validate configuration values before use

3. **Hot-Reloading**

   - Enable hot-reloading during development
   - Use hot-reloading for quick UI iterations
   - Test configuration changes without restarting

4. **Thread Safety**
   - Always use proper locking when accessing configurations
   - Keep lock durations as short as possible
   - Consider using clone() for frequently accessed values

## Troubleshooting

### Common Issues

1. **Configuration Not Loading**

   - Check file path is correct
   - Verify RON syntax is valid
   - Ensure file permissions are correct

2. **Hot-Reloading Not Working**

   - Check file system permissions
   - Verify hot-reloading is enabled
   - Ensure file path is being watched

3. **Type Errors**
   - Verify configuration structure matches RON file
   - Check for missing or extra fields
   - Ensure proper type conversions

### Debug Tips

1. Enable debug logging:

```rust
std::env::set_var("RUST_LOG", "debug");
env_logger::init();
```

2. Print configuration content:

```rust
println!("Config: {:#?}", config);
```

3. Verify file watching:

```rust
config_loader.watch_config("config.ron")?;
println!("Watching: {:?}", config_loader.watched_paths());
```

## Advanced Usage

### Configuration Dependencies

```rust
// Load configurations in order
config_loader.load_config("configs/base_config.ron")?;
config_loader.load_config("configs/ui_config.ron")?;
config_loader.load_config("configs/theme_config.ron")?;
```

### Dynamic Configuration Generation

```rust
let generated_config = UIConfig {
    text: TextConfig {
        default_font: "Dynamic".to_string(),
        font_size: calculate_optimal_font_size(),
        ..Default::default()
    },
    ..Default::default()
};

config_manager.register(generated_config);
```

### Configuration Events

```rust
// Register for configuration updates
let (tx, rx) = std::sync::mpsc::channel();
config_manager.on_update("ui", move |_| {
    tx.send("UI config updated").unwrap();
});
```

Remember to check the [Technical Documentation](../technical/Configuration.md) for more detailed information about the configuration system.
