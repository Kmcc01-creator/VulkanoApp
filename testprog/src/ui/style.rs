use glam::Vec4;

#[derive(Debug, Clone)]
pub struct Style {
    pub background_color: Vec4,
    pub border_color: Vec4,
    pub border_width: f32,
    pub corner_radius: f32,
    pub text_color: Vec4,
    pub title_bar_color: Option<Vec4>,
    pub padding: Padding,
}

#[derive(Debug, Clone, Copy)]
pub struct Padding {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Default for Padding {
    fn default() -> Self {
        Self {
            left: 5.0,
            right: 5.0,
            top: 5.0,
            bottom: 5.0,
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background_color: Vec4::new(0.2, 0.2, 0.2, 1.0),
            border_color: Vec4::new(0.3, 0.3, 0.3, 1.0),
            border_width: 1.0,
            corner_radius: 0.0,
            text_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            title_bar_color: None,
            padding: Padding::default(),
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

    pub fn with_text_color(mut self, color: Vec4) -> Self {
        self.text_color = color;
        self
    }

    pub fn with_title_bar_color(mut self, color: Vec4) -> Self {
        self.title_bar_color = Some(color);
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }
}

impl Padding {
    pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn uniform(value: f32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    pub fn horizontal(mut self, value: f32) -> Self {
        self.left = value;
        self.right = value;
        self
    }

    pub fn vertical(mut self, value: f32) -> Self {
        self.top = value;
        self.bottom = value;
        self
    }
}
