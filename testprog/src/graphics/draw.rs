use glam::{Vec2, Vec4};

pub struct DrawColor(pub Vec4);

impl From<Vec4> for DrawColor {
    fn from(color: Vec4) -> Self {
        DrawColor(color)
    }
}

pub struct DrawParams {
    pub position: Vec2,
    pub color: DrawColor,
    pub font_size: f32,
}

impl Default for DrawParams {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            color: DrawColor(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            font_size: 14.0,
        }
    }
}

impl DrawParams {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    pub fn with_color(mut self, color: impl Into<DrawColor>) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
}
