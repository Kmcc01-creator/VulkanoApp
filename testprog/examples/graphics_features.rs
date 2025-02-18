use glam::Vec4;
use testprog::core::error::Error;
use testprog::graphics::{
    self, vertex_attribute, vertex_binding, GraphicsConfigBuilder, PerformanceManager,
    PipelineBuilder, Vertex2D, POSITION2_FORMAT,
};
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::swapchain::{create_surface_from_winit, Surface};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create instance and window
    let instance = Instance::new(InstanceCreateInfo::default(), None)?;
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Graphics Features Example")
        .build(&event_loop)
        .map_err(|e| Error::WindowCreation(e.to_string()))?;

    // Create surface
    let surface = create_surface_from_winit(instance.clone(), window)?;

    // Create graphics configuration
    let config = GraphicsConfigBuilder::new()
        .max_frames_in_flight(2)
        .msaa_samples(4)
        .texture_memory_budget(512)
        .camera_settings(
            std::f32::consts::PI / 4.0,    // FOV
            0.1,                           // Near plane
            1000.0,                        // Far plane
            Vec4::new(0.1, 0.1, 0.1, 1.0), // Clear color
        )
        .build();

    // Select physical device and create logical device
    let (device, mut queues) = {
        let physical = instance
            .enumerate_physical_devices()?
            .next()
            .ok_or_else(|| {
                Error::GraphicsInitialization("No physical device available".to_string())
            })?;
        graphics::create_logical_device(physical, surface.clone())?
    };

    let queue = queues.next().unwrap();

    // Create renderer with configuration
    let mut renderer = graphics::RenderContext::new(queue.clone(), surface, config)?;

    // Initialize performance manager
    let mut perf_manager = PerformanceManager::new(
        device.clone(),
        queue.clone(),
        4, // Number of worker threads
    )?;

    // Create vertex data
    let vertices = vec![
        Vertex2D {
            position: [-0.5, -0.5],
        },
        Vertex2D {
            position: [0.5, -0.5],
        },
        Vertex2D {
            position: [0.0, 0.5],
        },
    ];

    // Load shaders
    let vertex_shader = graphics::shader::load_vertex_shader(device.clone())?;
    let fragment_shader = graphics::shader::load_fragment_shader(device.clone())?;

    // Create pipeline layout and render pass
    let layout = renderer.create_pipeline_layout()?;
    let render_pass = renderer.create_render_pass()?;

    // Create pipeline with builder
    let pipeline = PipelineBuilder::new(vertex_shader, fragment_shader)
        .vertex_binding(vertex_binding(0, std::mem::size_of::<Vertex2D>() as u32))
        .vertex_attribute(vertex_attribute(0, 0).format(POSITION2_FORMAT).offset(0))
        .build(&mut renderer.pipeline_manager(), layout, render_pass)?;

    // Create vertex buffer
    let vertex_buffer = renderer.create_vertex_buffer(&vertices)?;

    // Record commands using thread pool
    let cmd_buffer = perf_manager.record_commands(CommandBufferUsage::OneTimeSubmit, |cmd| {
        cmd.bind_pipeline_graphics(pipeline.clone())?;
        cmd.bind_vertex_buffers(0, vertex_buffer.clone())?;
        cmd.draw(vertices.len() as u32, 1, 0, 0)?;
        Ok(())
    })?;

    // Submit commands
    perf_manager.submit_commands(cmd_buffer)?;

    // Use batch renderer for multiple similar draw calls
    let mut batch_renderer = perf_manager.batch_renderer();
    batch_renderer.add_draw_call(pipeline.clone(), &vertices)?;
    batch_renderer.flush()?;

    // Get performance stats
    let stats = perf_manager.get_stats();
    println!("Command Pool Stats: {:?}", stats.command_pool_stats);
    println!("Batch Stats: {:?}", stats.batch_stats);
    println!("Loader Stats: {:?}", stats.loader_stats);

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                // Handle resize here if needed
            }
            Event::MainEventsCleared => {
                // Update renderer
                if renderer.begin_frame().is_ok() {
                    // Handle rendering here
                    let _ = renderer.end_frame();
                }
            }
            _ => (),
        }
    });
}
