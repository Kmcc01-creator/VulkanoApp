use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Event, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

#[derive(Default)]
struct InputState {
    mouse_pos: PhysicalPosition<f64>,
    left_button: bool,
    right_button: bool,
    window_focused: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the event loop
    let event_loop = EventLoop::new()?;

    // Create a basic window
    let window = Window::new(&event_loop)?;
    window.set_title("Input Handling Example");
    window.set_inner_size(LogicalSize::new(800, 600));

    // Center window
    if let Some(monitor) = window.current_monitor() {
        let monitor_size = monitor.size();
        let window_size = window.outer_size();
        let x = (monitor_size.width - window_size.width) / 2;
        let y = (monitor_size.height - window_size.height) / 2;
        window.set_outer_position(PhysicalPosition::new(x, y));
    }

    println!("Move mouse and click inside window to see input events");
    println!("Press Escape to exit");

    let mut input = InputState::default();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::CursorMoved { position, .. } => {
                    input.mouse_pos = position;
                    println!("Mouse position: ({:.1}, {:.1})", position.x, position.y);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    let pressed = state == ElementState::Pressed;
                    match button {
                        MouseButton::Left => {
                            input.left_button = pressed;
                            println!(
                                "Left button: {}",
                                if pressed { "pressed" } else { "released" }
                            );
                        }
                        MouseButton::Right => {
                            input.right_button = pressed;
                            println!(
                                "Right button: {}",
                                if pressed { "pressed" } else { "released" }
                            );
                        }
                        _ => (),
                    }
                }
                WindowEvent::Focused(focused) => {
                    input.window_focused = focused;
                    println!("Window focused: {}", focused);
                }
                WindowEvent::KeyboardInput {
                    input: key_input, ..
                } => {
                    use winit::event::VirtualKeyCode;
                    if let Some(keycode) = key_input.virtual_keycode {
                        if keycode == VirtualKeyCode::Escape {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                // Update window title with input state
                let title = format!(
                    "Mouse ({:.0}, {:.0}) - Left: {} Right: {} - Focused: {}",
                    input.mouse_pos.x,
                    input.mouse_pos.y,
                    input.left_button,
                    input.right_button,
                    input.window_focused
                );
                window.set_title(&title);
            }
            _ => (),
        }
    });
}
