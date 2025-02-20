mod atlas;
mod layout;
mod picking;
mod vertex;

pub use atlas::FontAtlas;
pub use layout::TextLayout;
pub use picking::TextPicker;
pub use vertex::TextVertex;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct TextElement {
    pub text: String,
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub scale: f32,
    pub element_id: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub advance: f32,
    pub bearing: [f32; 2],
    pub size: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub rect: Rect,
    pub element_id: u32,
}
