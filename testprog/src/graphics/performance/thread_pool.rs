use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use vulkano::command_buffer::{
    allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    PrimaryAutoCommandBuffer,
};
use vulkano::device::{Device, Queue};
use vulkano::sync::GpuFuture;

use crate::core::error::Error;

type CommandBuilder =
    AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>;
type CommandRecordFn = Box<dyn FnOnce(&mut CommandBuilder) -> Result<(), Error> + Send + 'static>;

enum ThreadMessage {
    Record {
        usage: CommandBufferUsage,
        recorder: CommandRecordFn,
        respond_to: Sender<Result<CommandBuilder, Error>>,
    },
    Shutdown,
}

pub struct CommandThreadPool {
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    command_sender: Sender<ThreadMessage>,
    command_allocator: Arc<StandardCommandBufferAllocator>,
    stats: Arc<Mutex<ThreadPoolStats>>,
    thread_handles: Vec<thread::JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub struct ThreadPoolStats {
    pub total_commands_recorded: u64,
    pub total_recording_time: std::time::Duration,
    pub active_threads: usize,
}

impl CommandThreadPool {
    pub fn new(
        device: Arc<Device>,
        graphics_queue: Arc<Queue>,
        num_threads: u32,
    ) -> Result<Self, Error> {
        let (sender, receiver) = bounded(32); // Buffer size for pending commands
        let command_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let stats = Arc::new(Mutex::new(ThreadPoolStats {
            total_commands_recorded: 0,
            total_recording_time: std::time::Duration::new(0, 0),
            active_threads: num_threads as usize,
        }));

        let mut thread_handles = Vec::new();

        // Spawn worker threads
        for _ in 0..num_threads {
            let thread_receiver = receiver.clone();
            let thread_device = device.clone();
            let thread_allocator = command_allocator.clone();
            let thread_stats = stats.clone();
            let queue_family = graphics_queue.queue_family();

            let handle = thread::Builder::new()
                .name("CommandThread".to_string())
                .spawn(move || {
                    Self::worker_thread(
                        thread_device,
                        thread_allocator,
                        thread_receiver,
                        thread_stats,
                        queue_family.index(),
                    )
                })
                .map_err(|e| Error::ThreadCreationError(e.to_string()))?;

            thread_handles.push(handle);
        }

        Ok(Self {
            device,
            graphics_queue,
            command_sender: sender,
            command_allocator,
            stats,
            thread_handles,
        })
    }

    fn worker_thread(
        device: Arc<Device>,
        allocator: Arc<StandardCommandBufferAllocator>,
        receiver: Receiver<ThreadMessage>,
        stats: Arc<Mutex<ThreadPoolStats>>,
        queue_family_index: u32,
    ) {
        while let Ok(message) = receiver.recv() {
            match message {
                ThreadMessage::Record {
                    usage,
                    recorder,
                    respond_to,
                } => {
                    let start_time = std::time::Instant::now();

                    let result = || -> Result<CommandBuilder, Error> {
                        let mut builder = AutoCommandBufferBuilder::primary(
                            &allocator,
                            queue_family_index,
                            usage,
                        )
                        .map_err(|e| {
                            Error::CommandRecordingError(format!(
                                "Failed to create command buffer: {}",
                                e
                            ))
                        })?;

                        recorder(&mut builder)?;

                        Ok(builder)
                    }();

                    let duration = start_time.elapsed();

                    // Update stats
                    if let Ok(mut stats) = stats.lock() {
                        stats.total_commands_recorded += 1;
                        stats.total_recording_time += duration;
                    }

                    let _ = respond_to.send(result);
                }
                ThreadMessage::Shutdown => break,
            }
        }
    }

    pub fn record_commands<F>(
        &self,
        usage: CommandBufferUsage,
        f: F,
    ) -> Result<CommandBuilder, Error>
    where
        F: FnOnce(&mut CommandBuilder) -> Result<(), Error> + Send + 'static,
    {
        let (sender, receiver) = bounded(1);

        self.command_sender
            .send(ThreadMessage::Record {
                usage,
                recorder: Box::new(f),
                respond_to: sender,
            })
            .map_err(|_| {
                Error::CommandRecordingError("Failed to send command recording request".into())
            })?;

        receiver
            .recv()
            .map_err(|_| Error::CommandRecordingError("Failed to receive command buffer".into()))?
    }

    pub fn submit_commands(&self, builder: CommandBuilder) -> Result<Box<dyn GpuFuture>, Error> {
        let command_buffer = builder.build().map_err(|e| {
            Error::CommandRecordingError(format!("Failed to build command buffer: {}", e))
        })?;

        Ok(Box::new(
            vulkano::sync::now(self.device.clone())
                .then_execute(self.graphics_queue.clone(), command_buffer)
                .map_err(|e| {
                    Error::CommandRecordingError(format!("Failed to submit command buffer: {}", e))
                })?,
        ))
    }

    pub fn get_stats(&self) -> ThreadPoolStats {
        self.stats.lock().unwrap().clone()
    }
}

impl Drop for CommandThreadPool {
    fn drop(&mut self) {
        // Signal all threads to shut down
        for _ in 0..self.stats.lock().unwrap().active_threads {
            let _ = self.command_sender.send(ThreadMessage::Shutdown);
        }

        // Wait for all threads to finish
        for handle in self.thread_handles.drain(..) {
            let _ = handle.join();
        }
    }
}
