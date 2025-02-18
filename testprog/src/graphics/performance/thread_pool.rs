use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::{Arc, Mutex};
use thread_pool::ThreadPool;

use vulkano::command_buffer::{
    allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    PrimaryAutoCommandBuffer,
};
use vulkano::device::{Device, Queue};
use vulkano::sync::GpuFuture;

use crate::core::error::Error;

type CommandRecordFn = Box<
    dyn FnOnce(&mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<(), Error>
        + Send
        + 'static,
>;

enum ThreadMessage {
    Record {
        usage: CommandBufferUsage,
        recorder: CommandRecordFn,
        respond_to: Sender<Result<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, Error>>,
    },
    Shutdown,
}

pub struct CommandThreadPool {
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    thread_pool: ThreadPool,
    command_sender: Sender<ThreadMessage>,
    command_allocator: Arc<StandardCommandBufferAllocator>,
    stats: Arc<Mutex<ThreadPoolStats>>,
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

        let pool = thread_pool::Builder::new()
            .num_threads(num_threads as usize)
            .build();

        // Spawn worker threads
        for _ in 0..num_threads {
            let thread_receiver = receiver.clone();
            let thread_device = device.clone();
            let thread_allocator = command_allocator.clone();
            let thread_stats = stats.clone();

            pool.execute(move || {
                Self::worker_thread(
                    thread_device,
                    thread_allocator,
                    thread_receiver,
                    thread_stats,
                )
            });
        }

        Ok(Self {
            device,
            graphics_queue,
            thread_pool: pool,
            command_sender: sender,
            command_allocator,
            stats,
        })
    }

    fn worker_thread(
        device: Arc<Device>,
        allocator: Arc<StandardCommandBufferAllocator>,
        receiver: Receiver<ThreadMessage>,
        stats: Arc<Mutex<ThreadPoolStats>>,
    ) {
        while let Ok(message) = receiver.recv() {
            match message {
                ThreadMessage::Record {
                    usage,
                    recorder,
                    respond_to,
                } => {
                    let start_time = std::time::Instant::now();

                    let result =
                        || -> Result<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, Error> {
                            let mut builder = AutoCommandBufferBuilder::primary(
                                &allocator,
                                device.active_queue_family().id(),
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
    ) -> Result<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, Error>
    where
        F: FnOnce(&mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<(), Error>
            + Send
            + 'static,
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

    pub fn submit_commands(
        &self,
        builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<Box<dyn GpuFuture>, Error> {
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
    }
}
