use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the event loop
    let event_loop = EventLoop::new()?;

    // Create a window using WindowBuilder
    let window = WindowBuilder::new()
        .with_title("Basic Window Example")
        .with_inner_size(LogicalSize::new(800, 600))
        .build(&event_loop)?;

    // Center the window on the primary monitor
    if let Some(monitor) = window.current_monitor() {
        let monitor_size = monitor.size();
        let window_size = window.outer_size();
        let x = (monitor_size.width - window_size.width) / 2;
        let y = (monitor_size.height - window_size.height) / 2;
        window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
    }

    println!("Window created - press Escape to exit");

    // Run the event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Window close requested");
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                use winit::event::VirtualKeyCode;
                if let Some(keycode) = input.virtual_keycode {
                    if keycode == VirtualKeyCode::Escape {
                        println!("Escape pressed");
                        *control_flow = ControlFlow::Exit;
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                println!("Window resized to: {}x{}", size.width, size.height);
            }
            _ => (),
        }
    });
}
