use crate::core::error::Error;
use glam::{Vec2, Vec3, Vec4};

/// A mock renderer for testing UI layout and functionality
pub struct MockRenderer {
    viewport_size: Vec2,
}

impl MockRenderer {
    pub fn new(viewport_size: Vec2) -> Self {
        Self { viewport_size }
    }

    pub fn draw_rect(&mut self, size: Vec2, position: Vec2, color: Vec4) -> Result<(), Error> {
        println!(
            "Drawing rect at ({}, {}) size ({}, {}) color ({:?})",
            position.x, position.y, size.x, size.y, color
        );
        Ok(())
    }

    pub fn draw_rect_outline(
        &mut self,
        size: Vec2,
        position: Vec2,
        width: f32,
        color: Vec4,
    ) -> Result<(), Error> {
        println!(
            "Drawing rect outline at ({}, {}) size ({}, {}) width {} color ({:?})",
            position.x, position.y, size.x, size.y, width, color
        );
        Ok(())
    }

    pub fn draw_text(&mut self, text: &str, position: Vec2, color: Vec4) -> Result<(), Error> {
        println!(
            "Drawing text '{}' at ({}, {}) color ({:?})",
            text, position.x, position.y, color
        );
        Ok(())
    }

    pub fn measure_text(&self, text: &str) -> Vec2 {
        Vec2::new(text.len() as f32 * 8.0, 14.0) // Simple approximation
    }

    pub fn world_to_screen(&self, world_pos: Vec3) -> Option<Vec2> {
        // Simple orthographic projection for testing
        Some(Vec2::new(
            world_pos.x + self.viewport_size.x * 0.5,
            -world_pos.y + self.viewport_size.y * 0.5,
        ))
    }
}
