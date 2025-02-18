use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use vulkano::device::Device;

use crate::core::error::Error;

pub struct AsyncLoader {
    device: Arc<Device>,
    resources: Arc<RwLock<Resources>>,
    loader_sender: Sender<LoaderMessage>,
    next_handle: Arc<Mutex<u64>>,
    stats: Arc<RwLock<LoaderStats>>,
}

#[derive(Debug, Clone)]
pub struct LoaderStats {
    pub total_loads: u64,
    pub successful_loads: u64,
    pub failed_loads: u64,
    pub pending_loads: u64,
    pub avg_load_time: std::time::Duration,
}

struct Resources {
    loaded: HashMap<u64, Box<dyn Resource>>,
    pending: HashMap<u64, ResourceStatus>,
}

#[derive(Clone, Copy, Debug)]
pub struct Handle<T>(u64, std::marker::PhantomData<T>);

enum LoaderMessage {
    Load {
        handle: u64,
        loader: Box<dyn FnOnce() -> Result<Box<dyn Resource>, Error> + Send>,
    },
    Shutdown,
}

#[derive(Clone)]
enum ResourceStatus {
    Loading,
    Loaded,
    Failed(String),
}

trait Resource: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: Send + Sync + 'static> Resource for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl AsyncLoader {
    pub fn new(device: Arc<Device>) -> Result<Self, Error> {
        let (sender, receiver) = bounded(32);
        let resources = Arc::new(RwLock::new(Resources {
            loaded: HashMap::new(),
            pending: HashMap::new(),
        }));
        let next_handle = Arc::new(Mutex::new(1));
        let stats = Arc::new(RwLock::new(LoaderStats {
            total_loads: 0,
            successful_loads: 0,
            failed_loads: 0,
            pending_loads: 0,
            avg_load_time: std::time::Duration::new(0, 0),
        }));

        // Start loader thread
        let thread_resources = resources.clone();
        let thread_stats = stats.clone();
        thread::spawn(move || {
            Self::loader_thread(receiver, thread_resources, thread_stats);
        });

        Ok(Self {
            device,
            resources,
            loader_sender: sender,
            next_handle,
            stats,
        })
    }

    fn loader_thread(
        receiver: Receiver<LoaderMessage>,
        resources: Arc<RwLock<Resources>>,
        stats: Arc<RwLock<LoaderStats>>,
    ) {
        while let Ok(message) = receiver.recv() {
            match message {
                LoaderMessage::Load { handle, loader } => {
                    let start_time = std::time::Instant::now();
                    let result = loader();
                    let duration = start_time.elapsed();

                    // Update resources
                    let mut resources = resources.write();
                    match result {
                        Ok(resource) => {
                            resources.loaded.insert(handle, resource);
                            resources.pending.insert(handle, ResourceStatus::Loaded);

                            // Update stats
                            let mut stats = stats.write();
                            stats.successful_loads += 1;
                            stats.pending_loads -= 1;
                            stats.avg_load_time = (stats.avg_load_time + duration) / 2;
                        }
                        Err(e) => {
                            resources
                                .pending
                                .insert(handle, ResourceStatus::Failed(e.to_string()));

                            // Update stats
                            let mut stats = stats.write();
                            stats.failed_loads += 1;
                            stats.pending_loads -= 1;
                        }
                    }
                }
                LoaderMessage::Shutdown => break,
            }
        }
    }

    pub fn load<T, F>(&self, loader: F) -> Result<Handle<T>, Error>
    where
        F: FnOnce() -> Result<T, Error> + Send + 'static,
        T: Send + Sync + 'static,
    {
        let handle = {
            let mut next_handle = self.next_handle.lock().unwrap();
            let handle = *next_handle;
            *next_handle += 1;
            handle
        };

        // Mark as pending
        {
            let mut resources = self.resources.write();
            resources.pending.insert(handle, ResourceStatus::Loading);

            let mut stats = self.stats.write();
            stats.total_loads += 1;
            stats.pending_loads += 1;
        }

        // Create wrapped loader that boxes the result
        let wrapped_loader = Box::new(move || -> Result<Box<dyn Resource>, Error> {
            loader().map(|r| Box::new(r) as Box<dyn Resource>)
        });

        // Send load message
        self.loader_sender
            .send(LoaderMessage::Load {
                handle,
                loader: wrapped_loader,
            })
            .map_err(|_| Error::ResourceCreation("Failed to send load request".into()))?;

        Ok(Handle(handle, std::marker::PhantomData))
    }

    pub fn get<T: 'static>(&self, handle: Handle<T>) -> Option<Arc<T>> {
        let resources = self.resources.read();
        resources.loaded.get(&handle.0).and_then(|resource| {
            resource
                .as_any()
                .downcast_ref::<T>()
                .map(|r| Arc::new(r.clone()))
        })
    }

    pub fn is_ready<T>(&self, handle: Handle<T>) -> bool {
        let resources = self.resources.read();
        matches!(
            resources.pending.get(&handle.0),
            Some(ResourceStatus::Loaded)
        )
    }

    pub fn get_status<T>(&self, handle: Handle<T>) -> Option<LoadStatus> {
        let resources = self.resources.read();
        resources.pending.get(&handle.0).map(|status| match status {
            ResourceStatus::Loading => LoadStatus::Loading,
            ResourceStatus::Loaded => LoadStatus::Ready,
            ResourceStatus::Failed(err) => LoadStatus::Failed(err.clone()),
        })
    }

    pub fn get_stats(&self) -> LoaderStats {
        self.stats.read().clone()
    }
}

#[derive(Debug, Clone)]
pub enum LoadStatus {
    Loading,
    Ready,
    Failed(String),
}

impl Drop for AsyncLoader {
    fn drop(&mut self) {
        let _ = self.loader_sender.send(LoaderMessage::Shutdown);
    }
}
