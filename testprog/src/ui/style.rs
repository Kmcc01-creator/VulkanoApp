use glam::Vec4;

#[derive(Debug, Clone)]
pub struct Style {
    pub background_color: Vec4,
    pub border_color: Vec4,
    pub border_width: f32,
    pub corner_radius: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background_color: Vec4::new(0.2, 0.2, 0.2, 1.0),
            border_color: Vec4::new(0.3, 0.3, 0.3, 1.0),
            border_width: 1.0,
            corner_radius: 0.0,
        }
    }
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_background_color(mut self, color: Vec4) -> Self {
        self.background_color = color;
        self
    }

    pub fn with_border_color(mut self, color: Vec4) -> Self {
        self.border_color = color;
        self
    }

    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
}
