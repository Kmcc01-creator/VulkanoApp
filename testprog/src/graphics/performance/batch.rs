use std::collections::HashMap;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::device::Device;
use vulkano::pipeline::GraphicsPipeline;

use crate::core::error::Error;

const MAX_BATCH_SIZE: usize = 10000;

pub struct BatchRenderer {
    device: Arc<Device>,
    batches: HashMap<BatchKey, DrawBatch>,
    stats: BatchStats,
}

#[derive(Debug, Clone)]
pub struct BatchStats {
    pub total_draw_calls: u64,
    pub batched_draw_calls: u64,
    pub batch_flushes: u64,
}

#[derive(Hash, Eq, PartialEq)]
struct BatchKey {
    pipeline: Arc<GraphicsPipeline>,
}

struct DrawBatch {
    vertices: Vec<u8>,
    vertex_size: usize,
    max_vertices: usize,
}

impl BatchRenderer {
    pub fn new(device: Arc<Device>) -> Result<Self, Error> {
        Ok(Self {
            device,
            batches: HashMap::new(),
            stats: BatchStats {
                total_draw_calls: 0,
                batched_draw_calls: 0,
                batch_flushes: 0,
            },
        })
    }

    pub fn add_draw_call<V>(
        &mut self,
        pipeline: Arc<GraphicsPipeline>,
        vertices: &[V],
    ) -> Result<(), Error> {
        let key = BatchKey { pipeline };
        let vertex_size = std::mem::size_of::<V>();

        self.stats.total_draw_calls += 1;

        let batch = self.batches.entry(key).or_insert_with(|| DrawBatch {
            vertices: Vec::new(),
            vertex_size,
            max_vertices: MAX_BATCH_SIZE,
        });

        // Check if we need to flush the batch
        if batch.vertices.len() + (vertices.len() * vertex_size) > batch.max_vertices * vertex_size
        {
            self.flush()?;
        }

        // Add vertices to batch
        let vertex_bytes = unsafe {
            std::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * vertex_size)
        };
        batch.vertices.extend_from_slice(vertex_bytes);
        self.stats.batched_draw_calls += 1;

        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        if self.batches.is_empty() {
            return Ok(());
        }

        let mut new_batches = HashMap::new();
        std::mem::swap(&mut self.batches, &mut new_batches);

        for (key, batch) in new_batches {
            if batch.vertices.is_empty() {
                continue;
            }

            // Create vertex buffer using StandardMemoryAllocator
            let allocator = vulkano::memory::allocator::StandardMemoryAllocator::new_default(
                self.device.clone(),
            );
            let buffer = Buffer::new_slice(
                &allocator,
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                vulkano::memory::allocator::AllocationCreateInfo {
                    memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_HOST
                        | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                batch.vertices.len() as u64,
            )
            .map_err(|e| {
                Error::ResourceCreation(format!("Failed to create vertex buffer: {}", e))
            })?;

            // Copy vertices to buffer
            buffer
                .write()
                .map_err(|e| {
                    Error::ResourceCreation(format!("Failed to write to vertex buffer: {}", e))
                })?
                .copy_from_slice(&batch.vertices);

            // Record draw command
            let vertices_count = batch.vertices.len() / batch.vertex_size;
            // Note: Command buffer recording would happen here in a real implementation
        }

        self.stats.batch_flushes += 1;
        Ok(())
    }

    pub fn get_stats(&self) -> BatchStats {
        self.stats.clone()
    }
}

// Helper trait for batch-compatible vertices
pub trait BatchVertex: Sized {
    fn get_binding_description(
    ) -> vulkano::pipeline::graphics::vertex_input::VertexInputBindingDescription;
    fn get_attribute_descriptions(
    ) -> Vec<vulkano::pipeline::graphics::vertex_input::VertexInputAttributeDescription>;
}

#[derive(Debug)]
pub struct BatchDrawCommand<V> {
    pipeline: Arc<GraphicsPipeline>,
    vertices: Vec<V>,
    vertex_count: u32,
}

impl<V> BatchDrawCommand<V> {
    pub fn new(pipeline: Arc<GraphicsPipeline>) -> Self {
        Self {
            pipeline,
            vertices: Vec::new(),
            vertex_count: 0,
        }
    }

    pub fn add_vertices(&mut self, vertices: &[V])
    where
        V: Clone,
    {
        self.vertices.extend_from_slice(vertices);
        self.vertex_count += vertices.len() as u32;
    }
}
