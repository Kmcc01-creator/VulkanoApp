use super::{Rect, Style};
use glam::Vec2;

#[derive(Debug, Clone, Default)]
pub struct Layout {
    pub margin: Vec2,
    pub padding: Vec2,
    pub min_size: Vec2,
    pub max_size: Option<Vec2>,
    pub position_type: PositionType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionType {
    Static,
    Relative(Vec2),
    Absolute(Vec2),
}

impl Default for PositionType {
    fn default() -> Self {
        Self::Static
    }
}

impl Layout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_margin(mut self, margin: Vec2) -> Self {
        self.margin = margin;
        self
    }

    pub fn with_padding(mut self, padding: Vec2) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    pub fn with_max_size(mut self, max_size: Vec2) -> Self {
        self.max_size = Some(max_size);
        self
    }

    pub fn with_position(mut self, position_type: PositionType) -> Self {
        self.position_type = position_type;
        self
    }

    pub fn calculate_bounds(&self, available_space: Rect, _style: &Style) -> Rect {
        let mut pos = available_space.position;
        let mut size = available_space.size;

        // Apply positioning
        match self.position_type {
            PositionType::Static => {
                pos += self.margin;
                size -= self.margin * 2.0;
            }
            PositionType::Relative(offset) => {
                pos += offset + self.margin;
                size -= self.margin * 2.0;
            }
            PositionType::Absolute(position) => {
                pos = position + self.margin;
            }
        }

        // Apply size constraints
        size = size.max(self.min_size);
        if let Some(max_size) = self.max_size {
            size = size.min(max_size);
        }

        Rect::new(pos, size)
    }

    pub fn calculate_content_area(&self, bounds: Rect) -> Rect {
        let content_pos = bounds.position + self.padding;
        let content_size = bounds.size - (self.padding * 2.0);

        Rect::new(content_pos, content_size)
    }
}
