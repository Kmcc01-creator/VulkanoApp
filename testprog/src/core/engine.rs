use super::error::Error;
use super::input::InputState;
use super::window::Window;
use crate::graphics::Graphics;
use crate::resource::ResourceManager;
use crate::scene::SceneManager;

/// The main engine struct that coordinates all systems
pub struct Engine {
    window: Window,
    graphics: Graphics,
    resources: ResourceManager,
    scene_manager: SceneManager,
    debug_input: bool,
}

impl Engine {
    /// Create a new engine instance
    pub fn new() -> Result<Self, Error> {
        let window = Window::new()?;
        let graphics = Graphics::new(&window)?;
        let resources = ResourceManager::new();
        let scene_manager = SceneManager::new();

        Ok(Self {
            window,
            graphics,
            resources,
            scene_manager,
            debug_input: false,
        })
    }

    /// Enable input debugging
    pub fn enable_input_debug(&mut self) {
        self.debug_input = true;
    }

    /// Run the main engine loop
    pub fn run(&mut self) -> Result<(), Error> {
        while !self.window.should_close() {
            self.update()?;
            self.render()?;
        }
        Ok(())
    }

    /// Update game logic
    fn update(&mut self) -> Result<(), Error> {
        // Update window and process events
        self.window.update();

        // Get input state
        let input = self.window.input();

        // Debug input if enabled
        if self.debug_input {
            let pos = input.mouse_position();
            let delta = input.mouse_delta();
            println!(
                "Mouse - Pos: ({:.1}, {:.1}), Delta: ({:.1}, {:.1}), Left: {}, Right: {}",
                pos.x,
                pos.y,
                delta.0,
                delta.1,
                input.is_mouse_button_pressed(winit::event::MouseButton::Left),
                input.is_mouse_button_pressed(winit::event::MouseButton::Right)
            );
        }

        // Update scene with input
        self.scene_manager.update(&self.resources);

        Ok(())
    }

    /// Render the current frame
    fn render(&mut self) -> Result<(), Error> {
        // Handle window resize
        if self.window.was_resized() {
            let (width, height) = self.window.size();
            self.graphics.update_viewport(width, height)?;
        }

        self.graphics.begin_frame()?;
        self.scene_manager.render(&mut self.graphics)?;
        self.graphics.end_frame()?;
        Ok(())
    }
}
