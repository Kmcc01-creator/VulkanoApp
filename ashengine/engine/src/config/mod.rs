use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Main configuration manager that handles all engine configurations
pub struct ConfigManager {
    configs: RwLock<HashMap<String, Box<dyn Config>>>,
}

/// Trait for configuration modules
pub trait Config: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn module_name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub text: TextConfig,
    pub layout: LayoutConfig,
    pub themes: HashMap<String, Theme>,
    pub active_theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    pub default_font: String,
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub sdf_settings: SDFSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDFSettings {
    pub smoothing: f32,
    pub thickness: f32,
    pub padding: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub padding: Padding,
    pub spacing: Spacing,
    pub alignment: Alignment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spacing {
    pub horizontal: f32,
    pub vertical: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub colors: HashMap<String, [f32; 4]>,
    pub fonts: HashMap<String, String>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
        }
    }

    pub fn register<T: Config + 'static>(&self, config: T) {
        let mut configs = self.configs.write().unwrap();
        configs.insert(config.module_name().to_string(), Box::new(config));
    }

    pub fn get<T: 'static>(&self, module_name: &str) -> Option<Arc<RwLock<T>>> {
        let configs = self.configs.read().unwrap();
        configs
            .get(module_name)
            .and_then(|config| config.as_any().downcast_ref::<Arc<RwLock<T>>>().cloned())
    }

    pub fn update<T: Config + 'static>(&self, module_name: &str, updater: impl FnOnce(&mut T)) {
        if let Some(mut configs) = self.configs.write().ok() {
            if let Some(config) = configs.get_mut(module_name) {
                if let Some(typed_config) = config.as_any_mut().downcast_mut::<T>() {
                    updater(typed_config);
                }
            }
        }
    }
}

impl UIConfig {
    pub fn new() -> Self {
        Self {
            text: TextConfig {
                default_font: "default".to_string(),
                font_size: 16.0,
                line_height: 1.2,
                letter_spacing: 0.0,
                sdf_settings: SDFSettings {
                    smoothing: 0.125,
                    thickness: 0.5,
                    padding: 4.0,
                },
            },
            layout: LayoutConfig {
                padding: Padding {
                    top: 5.0,
                    right: 5.0,
                    bottom: 5.0,
                    left: 5.0,
                },
                spacing: Spacing {
                    horizontal: 10.0,
                    vertical: 10.0,
                },
                alignment: Alignment::Left,
            },
            themes: HashMap::new(),
            active_theme: "default".to_string(),
        }
    }
}

impl Config for UIConfig {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn module_name(&self) -> &str {
        "ui"
    }
}
