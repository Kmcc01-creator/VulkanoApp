use std::error::Error;
use std::path::Path;
use testprog::core::{Engine, Window};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let mut window = Window::new(&event_loop)?;
    let mut engine = Engine::new();

    // Initialize graphics
    let renderer = window.create_render_context()?;
    engine.initialize_graphics(renderer)?;

    // Initialize hot reload system
    engine.initialize_hot_reload()?;

    // Add a custom reload handler for shader files
    engine.add_hot_reload_handler(|path| {
        if path.extension().map_or(false, |ext| ext == "glsl") {
            println!("Reloading shader: {:?}", path);
            // In a real implementation, you would reload the shader here
        }
        Ok(())
    })?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                // Update engine state
                if let Err(e) = engine.update() {
                    eprintln!("Error updating engine: {}", e);
                }

                // Request redraw
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if let Err(e) = engine.render() {
                    eprintln!("Error rendering frame: {}", e);
                }
            }
            Event::WindowEvent { event, .. } => {
                engine.handle_input(&event);
            }
            _ => {}
        }
    });
}
