use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

// Messages we'll send between threads
#[derive(Debug)]
enum WindowMessage {
    Resize(u32, u32),
    Move(i32, i32),
    Focus(bool),
    Close,
}

struct WindowState {
    size: (u32, u32),
    position: (i32, i32),
    focused: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create message channels
    let (tx, rx) = channel::<WindowMessage>();

    // Create event loop
    let event_loop = EventLoop::new()?;

    // Create window
    let window = Window::new(&event_loop)?;
    window.set_title("Event Loop Example");
    window.set_inner_size(LogicalSize::new(800, 600));

    // Center window
    if let Some(monitor) = window.current_monitor() {
        let monitor_size = monitor.size();
        let window_size = window.outer_size();
        let x = (monitor_size.width - window_size.width) / 2;
        let y = (monitor_size.height - window_size.height) / 2;
        window.set_outer_position(PhysicalPosition::new(x, y));
    }

    println!("Window created - Events will be printed below");
    println!("Press Escape to exit");

    // Create window state
    let mut state = WindowState {
        size: (800, 600),
        position: (0, 0),
        focused: true,
    };

    // Spawn state handling thread
    let state_thread = thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            match msg {
                WindowMessage::Resize(w, h) => println!("Window resized to {}x{}", w, h),
                WindowMessage::Move(x, y) => println!("Window moved to ({}, {})", x, y),
                WindowMessage::Focus(focused) => println!("Window focus: {}", focused),
                WindowMessage::Close => {
                    println!("Window closing");
                    break;
                }
            }
        }
    });

    // Clone sender for event loop
    let tx = tx.clone();

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    let _ = tx.send(WindowMessage::Close);
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    state.size = (size.width, size.height);
                    let _ = tx.send(WindowMessage::Resize(size.width, size.height));
                }
                WindowEvent::Moved(position) => {
                    state.position = (position.x, position.y);
                    let _ = tx.send(WindowMessage::Move(position.x, position.y));
                }
                WindowEvent::Focused(focused) => {
                    state.focused = focused;
                    let _ = tx.send(WindowMessage::Focus(focused));
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    use winit::event::VirtualKeyCode;
                    if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                        if input.state == ElementState::Pressed {
                            let _ = tx.send(WindowMessage::Close);
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                // Update window title with state
                let title = format!(
                    "Size: {}x{} - Pos: ({},{}) - Focused: {}",
                    state.size.0, state.size.1, state.position.0, state.position.1, state.focused
                );
                window.set_title(&title);
            }
            _ => (),
        }
    });

    // Wait for state thread to finish
    let _ = state_thread.join();
    Ok(())
}
