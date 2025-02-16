use super::mock_renderer::MockRenderer;
use super::{Rect, Style};
use crate::core::error::Error;
use glam::{Vec2, Vec3, Vec4};
use std::fmt::Debug;
use std::sync::Arc;

pub trait Widget: Debug + Send + Sync {
    fn render(&self, bounds: Rect, style: &Style, renderer: &mut MockRenderer)
        -> Result<(), Error>;
    fn update(&mut self) {}
    fn preferred_size(&self) -> Option<Vec2> {
        None
    }
    fn world_position(&self) -> Option<Vec3> {
        None
    }
    fn set_world_position(&mut self, _position: Vec3) {}
    fn is_world_space(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Button {
    text: Arc<str>,
    pressed: bool,
    size: Vec2,
}

impl Button {
    pub fn new<S: Into<Arc<str>>>(text: S) -> Self {
        Self {
            text: text.into(),
            pressed: false,
            size: Vec2::new(100.0, 30.0), // Default size
        }
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
}

impl Widget for Button {
    fn render(
        &self,
        bounds: Rect,
        style: &Style,
        renderer: &mut MockRenderer,
    ) -> Result<(), Error> {
        // Draw button background
        renderer.draw_rect(bounds.size, bounds.position, style.background_color)?;

        // Draw border if specified
        if style.border_width > 0.0 {
            renderer.draw_rect_outline(
                bounds.size,
                bounds.position,
                style.border_width,
                style.border_color,
            )?;
        }

        // Draw text centered in button
        let text_size = renderer.measure_text(&self.text);
        let text_pos = bounds.position
            + Vec2::new(
                (bounds.size.x - text_size.x) * 0.5,
                (bounds.size.y - text_size.y) * 0.5,
            );
        renderer.draw_text(&self.text, text_pos, style.text_color)?;

        Ok(())
    }

    fn preferred_size(&self) -> Option<Vec2> {
        Some(self.size)
    }
}

#[derive(Debug)]
pub struct Window {
    title: Arc<str>,
    dragging: bool,
    drag_start: Option<Vec2>,
    size: Vec2,
}

impl Window {
    pub fn new<S: Into<Arc<str>>>(title: S) -> Self {
        Self {
            title: title.into(),
            dragging: false,
            drag_start: None,
            size: Vec2::new(300.0, 200.0), // Default window size
        }
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn start_drag(&mut self, mouse_pos: Vec2) {
        self.dragging = true;
        self.drag_start = Some(mouse_pos);
    }

    pub fn end_drag(&mut self) {
        self.dragging = false;
        self.drag_start = None;
    }
}

impl Widget for Window {
    fn render(
        &self,
        bounds: Rect,
        style: &Style,
        renderer: &mut MockRenderer,
    ) -> Result<(), Error> {
        // Draw window background
        renderer.draw_rect(bounds.size, bounds.position, style.background_color)?;

        // Draw title bar
        let title_height = 25.0;
        let title_bounds = Rect::new(bounds.position, Vec2::new(bounds.size.x, title_height));

        let title_color = style
            .title_bar_color
            .unwrap_or(Vec4::new(0.3, 0.3, 0.3, 1.0));
        renderer.draw_rect(title_bounds.size, title_bounds.position, title_color)?;

        // Draw title text
        let text_pos = title_bounds.position + Vec2::new(5.0, 5.0);
        renderer.draw_text(&self.title, text_pos, style.text_color)?;

        // Draw border
        if style.border_width > 0.0 {
            renderer.draw_rect_outline(
                bounds.size,
                bounds.position,
                style.border_width,
                style.border_color,
            )?;
        }

        Ok(())
    }

    fn preferred_size(&self) -> Option<Vec2> {
        Some(self.size)
    }
}

#[derive(Debug)]
pub struct WorldSpaceWidget {
    widget: Arc<dyn Widget>,
    world_pos: Vec3,
    size: Vec2,
}

impl WorldSpaceWidget {
    pub fn new(widget: impl Widget + 'static, position: Vec3, size: Vec2) -> Self {
        Self {
            widget: Arc::new(widget),
            world_pos: position,
            size,
        }
    }
}

impl Widget for WorldSpaceWidget {
    fn render(
        &self,
        bounds: Rect,
        style: &Style,
        renderer: &mut MockRenderer,
    ) -> Result<(), Error> {
        if let Some(screen_pos) = renderer.world_to_screen(self.world_pos) {
            // Adjust bounds to screen position while maintaining size
            let screen_bounds = Rect::new(screen_pos, self.size);
            self.widget.render(screen_bounds, style, renderer)?;
        }

        Ok(())
    }

    fn world_position(&self) -> Option<Vec3> {
        Some(self.world_pos)
    }

    fn set_world_position(&mut self, position: Vec3) {
        self.world_pos = position;
    }

    fn is_world_space(&self) -> bool {
        true
    }

    fn preferred_size(&self) -> Option<Vec2> {
        Some(self.size)
    }
}
