use crate::core::error::Error;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};

pub struct HotReloader {
    watcher: RecommendedWatcher,
    rx: Receiver<Result<Event, notify::Error>>,
    watched_paths: Vec<PathBuf>,
    reload_handlers: Vec<Box<dyn Fn(&Path) -> Result<(), Error> + Send>>,
}

impl HotReloader {
    pub fn new() -> Result<Self, Error> {
        let (tx, rx) = channel();

        let watcher = notify::recommended_watcher(move |res| {
            tx.send(res).unwrap_or_default();
        })
        .map_err(|e| Error::Other(format!("Failed to create watcher: {}", e)))?;

        Ok(Self {
            watcher,
            rx,
            watched_paths: Vec::new(),
            reload_handlers: Vec::new(),
        })
    }

    pub fn watch<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let path = path.as_ref().to_path_buf();
        self.watcher
            .watch(&path, RecursiveMode::Recursive)
            .map_err(|e| Error::Other(format!("Failed to watch path: {}", e)))?;
        self.watched_paths.push(path);
        Ok(())
    }

    pub fn add_reload_handler<F>(&mut self, handler: F)
    where
        F: Fn(&Path) -> Result<(), Error> + Send + 'static,
    {
        self.reload_handlers.push(Box::new(handler));
    }

    pub fn update(&mut self) -> Result<(), Error> {
        // Process any pending events
        while let Ok(Ok(event)) = self.rx.try_recv() {
            if let Some(path) = event.paths.first() {
                // Only trigger reload for modify events
                if event.kind.is_modify() {
                    for handler in &self.reload_handlers {
                        handler(path)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Drop for HotReloader {
    fn drop(&mut self) {
        for path in &self.watched_paths {
            if let Err(e) = self.watcher.unwatch(path) {
                eprintln!("Failed to unwatch path: {}", e);
            }
        }
    }
}
