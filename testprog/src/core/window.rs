use super::error::Error;
use super::input::InputState;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Event, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::window::Window as WinitWindow;

#[derive(Debug)]
enum WindowMessage {
    Resize(u32, u32),
    Move(i32, i32),
    Focus(bool),
    Close,
    MouseInput(MouseButton, ElementState),
    CursorMoved(PhysicalPosition<f64>),
}

struct WindowState {
    size: (u32, u32),
    position: (i32, i32),
    focused: bool,
}

pub struct Window {
    window: Arc<WinitWindow>,
    event_loop: Option<EventLoop<()>>,
    state: WindowState,
    input: InputState,
    should_close: bool,
    message_sender: Sender<WindowMessage>,
    message_receiver: Receiver<WindowMessage>,
}

impl Window {
    pub fn new() -> Result<Self, Error> {
        let event_loop = EventLoopBuilder::new().build();

        let window = Arc::new(
            winit::window::WindowBuilder::new()
                .with_title("Game Engine")
                .with_inner_size(LogicalSize::new(800, 600))
                .build(&event_loop)
                .map_err(|e| Error::WindowCreation(e.to_string()))?,
        );

        if let Some(monitor) = window.current_monitor() {
            let monitor_size = monitor.size();
            let window_size = window.outer_size();
            let x = (monitor_size.width - window_size.width) / 2;
            let y = (monitor_size.height - window_size.height) / 2;
            window.set_outer_position(PhysicalPosition::new(x, y));
        }

        let (sender, receiver) = channel();

        let state = WindowState {
            size: (800, 600),
            position: (0, 0),
            focused: true,
        };

        Ok(Self {
            window,
            event_loop: Some(event_loop),
            state,
            input: InputState::new(),
            should_close: false,
            message_sender: sender,
            message_receiver: receiver,
        })
    }

    pub fn run_event_loop<F>(&mut self, _callback: F) -> !
    where
        F: FnMut(&mut Self) + 'static,
    {
        let event_loop = self.event_loop.take().expect("Event loop already taken");

        let sender = self.message_sender.clone();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        let _ = sender.send(WindowMessage::Close);
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(size) => {
                        let _ = sender.send(WindowMessage::Resize(size.width, size.height));
                    }
                    WindowEvent::Moved(position) => {
                        let _ = sender.send(WindowMessage::Move(position.x, position.y));
                    }
                    WindowEvent::Focused(focused) => {
                        let _ = sender.send(WindowMessage::Focus(focused));
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        let _ = sender.send(WindowMessage::MouseInput(button, state));
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let _ = sender.send(WindowMessage::CursorMoved(position));
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    // Nothing needed here since we'll process messages in handle_events
                }
                _ => (),
            }
        })
    }

    pub fn handle_events(&mut self) -> Result<(), Error> {
        while let Ok(msg) = self.message_receiver.try_recv() {
            match msg {
                WindowMessage::Resize(w, h) => {
                    self.state.size = (w, h);
                }
                WindowMessage::Move(x, y) => {
                    self.state.position = (x, y);
                }
                WindowMessage::Focus(focused) => {
                    self.state.focused = focused;
                }
                WindowMessage::Close => {
                    self.should_close = true;
                }
                WindowMessage::MouseInput(button, state) => {
                    self.input.update_mouse_button(button, state);
                }
                WindowMessage::CursorMoved(position) => {
                    self.input.update_mouse_position(position);
                }
            }
        }
        Ok(())
    }

    pub fn should_close(&self) -> bool {
        self.should_close
    }

    pub fn request_close(&mut self) {
        self.should_close = true;
    }

    pub fn size(&self) -> (u32, u32) {
        self.state.size
    }

    pub fn was_resized(&mut self) -> bool {
        let resized = self.state.focused;
        self.state.focused = false;
        resized
    }

    pub fn raw_window_handle(&self) -> Arc<WinitWindow> {
        self.window.clone()
    }

    pub fn input(&self) -> &InputState {
        &self.input
    }

    pub fn get_required_extensions(&self) -> Result<vulkano::instance::InstanceExtensions, Error> {
        let event_loop = self
            .event_loop
            .as_ref()
            .ok_or_else(|| Error::WindowCreation("Event loop not available".into()))?;

        Ok(vulkano::swapchain::Surface::required_extensions(event_loop))
    }

    pub fn create_surface(
        &self,
        instance: Arc<vulkano::instance::Instance>,
    ) -> Result<Arc<vulkano::swapchain::Surface>, Error> {
        vulkano::swapchain::Surface::from_window(instance, self.window.clone())
            .map_err(|e| Error::WindowCreation(format!("Failed to create surface: {}", e)))
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = self.event_loop.take();
    }
}
