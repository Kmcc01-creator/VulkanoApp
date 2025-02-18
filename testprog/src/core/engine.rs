use crate::core::error::Error;
use crate::core::hot_reload::HotReloader;
use crate::core::input::InputState;
use crate::graphics::renderer::RenderContext;
use crate::ui::UI;

pub struct Engine {
    input: InputState,
    renderer: Option<RenderContext>,
    ui: Option<UI>,
    hot_reloader: Option<HotReloader>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            input: InputState::new(),
            renderer: None,
            ui: None,
            hot_reloader: None,
        }
    }

    pub fn initialize_graphics(&mut self, renderer: RenderContext) -> Result<(), Error> {
        let viewport_size = renderer.viewport_size();
        self.ui = Some(UI::new(viewport_size));
        self.renderer = Some(renderer);
        Ok(())
    }

    pub fn initialize_hot_reload(&mut self) -> Result<(), Error> {
        let mut hot_reloader = HotReloader::new()?;

        // Watch src directory for changes
        hot_reloader.watch("src")?;

        // Add default reload handlers
        hot_reloader.add_reload_handler(|path| {
            println!("Detected change in: {:?}", path);
            Ok(())
        });

        self.hot_reloader = Some(hot_reloader);
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), Error> {
        // Update input state
        self.input.update();

        // Update UI if initialized
        if let Some(ui) = &mut self.ui {
            ui.update();
        }

        // Update hot reloader if initialized
        if let Some(hot_reloader) = &mut self.hot_reloader {
            hot_reloader.update()?;
        }

        Ok(())
    }

    pub fn render(&mut self) -> Result<(), Error> {
        if let Some(renderer) = &mut self.renderer {
            renderer.begin_frame()?;

            // Render UI if initialized
            if let Some(ui) = &mut self.ui {
                ui.render()?;
            }

            renderer.end_frame()?;
        }

        Ok(())
    }

    pub fn handle_input(&mut self, event: &winit::event::WindowEvent) {
        self.input.handle_event(event);

        // Forward relevant input events to UI
        if let Some(ui) = &mut self.ui {
            match event {
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    let pos = glam::Vec2::new(position.x as f32, position.y as f32);
                    ui.handle_hover(pos);
                }
                winit::event::WindowEvent::MouseInput {
                    state: winit::event::ElementState::Pressed,
                    ..
                } => {
                    if let Some(pos) = self.input.cursor_position() {
                        ui.handle_click(pos);
                    }
                }
                winit::event::WindowEvent::ReceivedCharacter(c) => {
                    ui.handle_key(*c);
                }
                winit::event::WindowEvent::Resized(size) => {
                    let new_size = glam::Vec2::new(size.width as f32, size.height as f32);
                    ui.resize(new_size);
                }
                _ => {}
            }
        }
    }

    pub fn input(&self) -> &InputState {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut InputState {
        &mut self.input
    }

    pub fn ui(&mut self) -> Option<&mut UI> {
        self.ui.as_mut()
    }

    pub fn add_hot_reload_handler<F>(&mut self, handler: F) -> Result<(), Error>
    where
        F: Fn(&std::path::Path) -> Result<(), Error> + Send + 'static,
    {
        if let Some(hot_reloader) = &mut self.hot_reloader {
            hot_reloader.add_reload_handler(handler);
            Ok(())
        } else {
            Err(Error::Other("Hot reloader not initialized".to_string()))
        }
    }
}
