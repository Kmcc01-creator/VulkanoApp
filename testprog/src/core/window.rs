use super::error::Error;
use super::input::InputState;
use std::sync::mpsc::{channel, Receiver, Sender};
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopBuilder};
use winit::window::Window as WinitWindow;

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

pub struct Window {
    window: WinitWindow,
    event_loop: Option<EventLoop<()>>,
    state: WindowState,
    input: InputState,
    should_close: bool,
    message_sender: Sender<WindowMessage>,
    message_receiver: Receiver<WindowMessage>,
}

impl Window {
    pub fn new() -> Result<Self, Error> {
        // Create event loop using builder pattern
        let event_loop = EventLoopBuilder::new()
            .build()
            .map_err(|e| Error::WindowCreation(e.to_string()))?;

        // Create window
        let window =
            WinitWindow::new(&event_loop).map_err(|e| Error::WindowCreation(e.to_string()))?;

        // Configure window
        window.set_title("Game Engine");
        window.set_inner_size(LogicalSize::new(800, 600));

        // Center window
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

    pub fn run_event_loop<F>(&mut self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(&mut Self),
    {
        let event_loop = self
            .event_loop
            .take()
            .ok_or_else(|| Error::WindowCreation("Event loop already taken".into()))?;

        let result = event_loop.run(|event, target| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    self.should_close = true;
                    target.exit();
                }
                WindowEvent::Resized(size) => {
                    self.state.size = (size.width, size.height);
                    let _ = self
                        .message_sender
                        .send(WindowMessage::Resize(size.width, size.height));
                }
                WindowEvent::Moved(position) => {
                    self.state.position = (position.x, position.y);
                    let _ = self
                        .message_sender
                        .send(WindowMessage::Move(position.x, position.y));
                }
                WindowEvent::Focused(focused) => {
                    self.state.focused = focused;
                    let _ = self.message_sender.send(WindowMessage::Focus(focused));
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    self.input.update_mouse_button(button, state);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.input.update_mouse_position(position);
                }
                _ => (),
            },
            Event::AboutToWait => {
                callback(self);

                if self.should_close {
                    target.exit();
                }
            }
            _ => (),
        });

        result.map_err(|e| Error::WindowCreation(e.to_string()))
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

    pub fn raw_window_handle(&self) -> &WinitWindow {
        &self.window
    }

    pub fn input(&self) -> &InputState {
        &self.input
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = self.event_loop.take();
    }
}
