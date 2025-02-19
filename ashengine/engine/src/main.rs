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
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct Engine {
    renderer: Renderer,
    shader_set: ShaderSet,
    swapchain: Swapchain,
    _context: Arc<VulkanContext>, // Keep context alive until everything else is dropped
}

impl Engine {
    fn new(window: &(impl HasRawWindowHandle + HasRawDisplayHandle)) -> Result<Self> {
        let context = Arc::new(VulkanContext::new(window)?);
        info!("Created Vulkan context");

        let dimensions = window.inner_size();
        let device = Arc::new(context.device().clone());
        let swapchain = Swapchain::new(
            context.physical_device(),
            device.clone(),
            context.instance(),
            context.surface_loader(),
            context.surface(),
            [dimensions.width, dimensions.height],
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

    let window = WindowBuilder::new()
        .with_title("Vulkan Engine")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();
    info!("Created window");

    let mut engine = Engine::new(&window)?;
    let mut minimized = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Window close requested");
                if let Err(e) = engine.cleanup() {
                    eprintln!("Failed to cleanup engine: {}", e);
                }
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if size.width == 0 || size.height == 0 {
                    minimized = true;
                } else {
                    minimized = false;
                    // TODO: Handle resizing
                }
            }
            Event::MainEventsCleared => {
                if !minimized {
                    match engine.render_frame(
                        engine._context.graphics_queue(),
                        engine._context.present_queue(),
                    ) {
                        Ok(needs_recreation) => {
                            if needs_recreation {
                                // TODO: Handle swapchain recreation
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
    });

    #[allow(unreachable_code)]
    Ok(())
}
