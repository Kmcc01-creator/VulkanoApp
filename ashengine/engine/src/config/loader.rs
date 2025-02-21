use super::{Config, ConfigManager, UIConfig};
use crate::error::{Result, VulkanError};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use ron::de::from_reader;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};

pub struct ConfigLoader {
    config_manager: Arc<ConfigManager>,
    watcher: Option<RecommendedWatcher>,
    config_paths: RwLock<Vec<PathBuf>>,
}

impl ConfigLoader {
    pub fn new(config_manager: Arc<ConfigManager>) -> Result<Self> {
        Ok(Self {
            config_manager,
            watcher: None,
            config_paths: RwLock::new(Vec::new()),
        })
    }

    /// Load a configuration file and register it with the config manager
    pub fn load_config<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|e| {
            VulkanError::ConfigurationError(format!("Failed to open config file: {}", e))
        })?;

        // Determine config type from file extension/name
        if path
            .file_name()
            .map(|n| n.to_string_lossy().contains("ui"))
            .unwrap_or(false)
        {
            let config: UIConfig = from_reader(file).map_err(|e| {
                VulkanError::ConfigurationError(format!("Failed to parse UI config: {}", e))
            })?;
            self.config_manager.register(config);
        }
        // Add more config types here as needed

        // Add to watched paths if hot-reloading is enabled
        if self.watcher.is_some() {
            self.config_paths.write().unwrap().push(path.to_owned());
        }

        Ok(())
    }

    /// Enable hot-reloading of configuration files
    pub fn enable_hot_reload(&mut self) -> Result<()> {
        let (tx, rx) = channel();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if matches!(event.kind, notify::EventKind::Modify(_)) {
                    let _ = tx.send(event);
                }
            }
        })
        .map_err(|e| VulkanError::ConfigurationError(format!("Failed to create watcher: {}", e)))?;

        // Watch all currently loaded config files
        for path in self.config_paths.read().unwrap().iter() {
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .map_err(|e| {
                    VulkanError::ConfigurationError(format!("Failed to watch config file: {}", e))
                })?;
        }

        let config_manager = Arc::clone(&self.config_manager);

        // Spawn thread to handle config reloading
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                if let notify::Event {
                    kind: notify::EventKind::Modify(_),
                    paths,
                    ..
                } = event
                {
                    for path in paths {
                        if let Ok(file) = File::open(&path) {
                            if path
                                .file_name()
                                .map(|n| n.to_string_lossy().contains("ui"))
                                .unwrap_or(false)
                            {
                                if let Ok(new_config) = from_reader::<_, UIConfig>(file) {
                                    config_manager.register(new_config);
                                }
                            }
                            // Add more config types here
                        }
                    }
                }
            }
        });

        self.watcher = Some(watcher);
        Ok(())
    }

    /// Add a new path to watch for changes
    pub fn watch_config<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        if let Some(watcher) = &mut self.watcher {
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .map_err(|e| {
                    VulkanError::ConfigurationError(format!("Failed to watch config file: {}", e))
                })?;
            self.config_paths.write().unwrap().push(path.to_owned());
        }
        Ok(())
    }
}

impl Drop for ConfigLoader {
    fn drop(&mut self) {
        if let Some(mut watcher) = self.watcher.take() {
            for path in self.config_paths.read().unwrap().iter() {
                let _ = watcher.unwatch(path);
            }
        }
    }
}
