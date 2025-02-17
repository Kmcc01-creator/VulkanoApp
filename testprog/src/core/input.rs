use std::collections::HashMap;
use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::MouseButton;

#[derive(Default, Clone)]
pub struct InputState {
    mouse_button_state: HashMap<MouseButton, bool>,
    mouse_position: PhysicalPosition<f64>,
    mouse_delta: (f64, f64),
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse_button_state: HashMap::new(),
            mouse_position: PhysicalPosition::new(0.0, 0.0),
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn update_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        self.mouse_button_state
            .insert(button, state == ElementState::Pressed);
    }

    pub fn update_mouse_position(&mut self, position: PhysicalPosition<f64>) {
        let old_position = self.mouse_position;
        self.mouse_position = position;
        self.mouse_delta = (position.x - old_position.x, position.y - old_position.y);
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        *self.mouse_button_state.get(&button).unwrap_or(&false)
    }

    pub fn mouse_position(&self) -> PhysicalPosition<f64> {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    pub fn clear_frame_state(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }
}
