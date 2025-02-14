use super::Rect;
use glam::Vec2;

#[derive(Debug, Clone, Default)]
pub struct Layout {
    pub margin: Vec2,
    pub padding: Vec2,
    pub min_size: Vec2,
    pub max_size: Option<Vec2>,
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

    pub fn calculate_size(&self, content_size: Vec2) -> Vec2 {
        let mut size = content_size + self.padding * 2.0 + self.margin * 2.0;
        size = size.max(self.min_size);
        if let Some(max_size) = self.max_size {
            size = size.min(max_size);
        }
        size
    }

    pub fn calculate_content_rect(&self, outer_rect: Rect) -> Rect {
        Rect {
            position: outer_rect.position + self.margin + self.padding,
            size: outer_rect.size - (self.margin + self.padding) * 2.0,
        }
    }
}
