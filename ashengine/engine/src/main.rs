mod commands;
mod context;
mod error;
mod pipeline;
mod render_pass;
mod renderer;
mod shader;
mod swapchain;

use ash::vk;
use context::VulkanContext;
use error::Result;
use renderer::Renderer;
use shader::ShaderSet;
use swapchain::Swapchain;

use log::info;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};

struct Engine {
    renderer: Renderer,
    shader_set: ShaderSet,
    swapchain: Swapchain,
    _context: Arc<VulkanContext>, // Keep context alive until everything else is dropped
}

impl Engine {
    fn new(
        window: &(impl HasRawWindowHandle + HasRawDisplayHandle),
        dimensions: [u32; 2],
    ) -> Result<Self> {
        let context = Arc::new(VulkanContext::new(window)?);
        info!("Created Vulkan context");
        let device = Arc::new(context.device().clone());
        let swapchain = Swapchain::new(
            context.physical_device(),
            device.clone(),
            context.instance(),
            context.surface_loader(),
            context.surface(),
            dimensions,
        )?;
        info!("Created swapchain");

        let shader_set = ShaderSet::new(
            device.clone(),
            "engine/shaders/vert.spv",
            "engine/shaders/frag.spv",
        )?;
        info!("Loaded shaders");

        let renderer = Renderer::new(
            device,
            &swapchain,
            &shader_set,
            0, // Using graphics queue family index 0
        )?;
        info!("Created renderer");

        Ok(Self {
            renderer,
            shader_set,
            swapchain,
            _context: context,
        })
    }

    fn render_frame(
        &mut self,
        graphics_queue: vk::Queue,
        present_queue: vk::Queue,
    ) -> Result<bool> {
        self.renderer
            .render_frame(&self.swapchain, graphics_queue, present_queue)
    }

    fn cleanup(&mut self) -> Result<()> {
        self.renderer.wait_idle()
    }

    fn recreate_swapchain(&mut self, dimensions: [u32; 2]) -> Result<()> {
        let ctx = &self._context;
        self.swapchain.recreate(
            ctx.physical_device(),
            Arc::new(ctx.device().clone()),
            ctx.instance(),
            ctx.surface_loader(),
            ctx.surface(),
            dimensions,
        )?;

        // We also need to recreate the renderer's resources that depend on the swapchain
        self.renderer = Renderer::new(
            Arc::new(ctx.device().clone()),
            &self.swapchain,
            &self.shader_set,
            0,
        )?;

        Ok(())
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    info!("Initializing Vulkan application");

    let event_loop = EventLoop::new();
    info!("Created event loop");
    let window_width = 800;
    let window_height = 600;
    let window = WindowBuilder::new()
        .with_title("Vulkan Engine")
        .with_inner_size(winit::dpi::LogicalSize::new(
            window_width as f64,
            window_height as f64,
        ))
        .build(&event_loop)
        .unwrap();
    info!("Created window");

    let mut engine = Engine::new(&window, [window_width, window_height])?;
    let mut minimized = false;

    // Since run() takes ownership of the event loop and never returns,
    // we need to handle cleanup before it runs
    let mut window_size = [window_width, window_height];
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    info!("Window close requested");
                    if let Err(e) = engine.cleanup() {
                        eprintln!("Failed to cleanup engine: {}", e);
                    }
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    let width = size.width as u32;
                    let height = size.height as u32;
                    if width == 0 || height == 0 {
                        minimized = true;
                    } else {
                        minimized = false;
                        window_size = [width, height];
                        if let Err(e) = engine.recreate_swapchain(window_size) {
                            eprintln!("Failed to recreate swapchain: {}", e);
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                if !minimized {
                    let ctx = &engine._context;
                    match engine.render_frame(ctx.graphics_queue(), ctx.present_queue()) {
                        Ok(needs_recreation) => {
                            if needs_recreation {
                                // For suboptimal/out-of-date swapchains, recreate with current window size
                                if let Err(e) = engine.recreate_swapchain(window_size) {
                                    eprintln!("Failed to recreate swapchain: {}", e);
                                    *control_flow = ControlFlow::Exit;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to render frame: {}", e);
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
            }
            _ => (),
        }
    })
}
