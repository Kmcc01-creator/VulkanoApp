# Configuration System

## Overview

The AshEngine configuration system provides a flexible, type-safe, and hot-reloadable way to manage engine settings. It supports multiple configuration types, automatic serialization/deserialization, and thread-safe access to configuration data.

## Core Components

### ConfigManager

The central component that manages all engine configurations. It provides:

- Thread-safe access to configurations
- Type-safe configuration retrieval
- Dynamic configuration updates
- Module-specific configuration isolation

```rust
let config_manager = Arc::new(ConfigManager::new());
let ui_config = config_manager.get::<UIConfig>("ui").unwrap();
```

### ConfigLoader

Handles loading and hot-reloading of configuration files:

- Supports multiple file formats (currently RON)
- Watches for file changes
- Automatically reloads modified configurations
- Validates configuration data

```rust
let mut config_loader = ConfigLoader::new(config_manager.clone())?;
config_loader.load_config("configs/ui_config.ron")?;
config_loader.enable_hot_reload()?;
```

## Configuration Types

### UI Configuration

The UI configuration system manages all aspects of the user interface:

#### Text Settings

```ron
text: (
    default_font: "Roboto",
    font_size: 16.0,
    line_height: 1.2,
    letter_spacing: 0.0,
    sdf_settings: (
        smoothing: 0.125,
        thickness: 0.5,
        padding: 4.0,
    ),
)
```

#### Layout Settings

```ron
layout: (
    padding: (
        top: 5.0,
        right: 5.0,
        bottom: 5.0,
        left: 5.0,
    ),
    spacing: (
        horizontal: 10.0,
        vertical: 10.0,
    ),
    alignment: Left,
)
```

#### Theme Support

```ron
themes: {
    "default": (
        colors: {
            "text": [1.0, 1.0, 1.0, 1.0],
            "background": [0.1, 0.1, 0.1, 1.0],
            "primary": [0.2, 0.6, 1.0, 1.0],
        },
        fonts: {
            "regular": "Roboto-Regular",
            "bold": "Roboto-Bold",
        },
    ),
}
```

## Usage Examples

### Basic Configuration Loading

```rust
use ashengine::config::{ConfigLoader, ConfigManager, UIConfig};

// Initialize configuration system
let config_manager = Arc::new(ConfigManager::new());
let mut config_loader = ConfigLoader::new(config_manager.clone())?;

// Load configuration
config_loader.load_config("configs/ui_config.ron")?;

// Access configuration
let ui_config = config_manager.get::<UIConfig>("ui").unwrap();
let font_size = ui_config.read().unwrap().text.font_size;
```

### Dynamic Updates

```rust
config_manager.update("ui", |config: &mut UIConfig| {
    config.text.font_size = 20.0;
    config.active_theme = "light".to_string();
});
```

### Hot-Reloading

```rust
// Enable hot-reloading for all loaded configurations
config_loader.enable_hot_reload()?;

// Add new configuration file to watch
config_loader.watch_config("configs/new_config.ron")?;
```

## Creating Custom Configurations

1. Define your configuration structure:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomConfig {
    pub setting1: String,
    pub setting2: f32,
}
```

2. Implement the Config trait:

```rust
impl Config for CustomConfig {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn module_name(&self) -> &str {
        "custom"
    }
}
```

3. Register with ConfigManager:

```rust
let custom_config = CustomConfig::new();
config_manager.register(custom_config);
```

## Thread Safety

- All configurations are wrapped in `Arc<RwLock<>>` for safe concurrent access
- Multiple readers can access configurations simultaneously
- Writers get exclusive access during updates
- Hot-reloading uses separate threads for file watching

## Best Practices

1. **Organization**

   - Keep configuration files in a dedicated directory
   - Use meaningful file names that reflect the configuration purpose
   - Group related settings together in the same configuration file

2. **Validation**

   - Implement validation for configuration values
   - Provide sensible defaults for all settings
   - Handle missing or invalid configuration gracefully

3. **Updates**

   - Use hot-reloading during development for quick iterations
   - Implement proper error handling for configuration updates
   - Consider using events to notify systems of configuration changes

4. **Documentation**
   - Document all configuration options
   - Provide example configuration files
   - Include comments in configuration files explaining each setting

## Error Handling

The configuration system provides detailed error information:

- File loading errors
- Parse errors
- Validation errors
- Hot-reload errors

Example error handling:

```rust
match config_loader.load_config("config.ron") {
    Ok(_) => println!("Configuration loaded successfully"),
    Err(e) => match e {
        VulkanError::ConfigurationError(msg) => {
            eprintln!("Configuration error: {}", msg);
        }
        _ => eprintln!("Unexpected error: {}", e),
    }
}
```

## Future Extensions

The configuration system is designed to be extensible:

- Support for additional file formats (JSON, YAML, etc.)
- Schema validation
- Configuration versioning
- Configuration inheritance
- Runtime configuration generation
- Configuration export/import
