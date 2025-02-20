use crate::error::Result;
use crate::text::{BoundingBox, FontAtlas, GlyphInfo, Rect, TextElement, TextVertex};
use ash::vk;
use std::sync::Arc;

pub struct TextLayout {
    vertices: Vec<TextVertex>,
    indices: Vec<u32>,
    bounding_boxes: Vec<BoundingBox>,
    vertex_buffer: Option<vk::Buffer>,
    index_buffer: Option<vk::Buffer>,
    bbox_buffer: Option<vk::Buffer>,
}

impl TextLayout {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            bounding_boxes: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
            bbox_buffer: None,
        }
    }

    pub fn layout_text(&mut self, text_elements: &[TextElement], atlas: &FontAtlas) {
        self.vertices.clear();
        self.indices.clear();
        self.bounding_boxes.clear();

        for element in text_elements {
            let mut cursor_x = element.position[0];
            let cursor_y = element.position[1];
            let mut element_width = 0.0;
            let mut element_height = 0.0;

            // First pass: calculate bounding box
            for c in element.text.chars() {
                if let Some(glyph) = atlas.get_glyph(c) {
                    element_width += glyph.metrics.advance * element.scale;
                    element_height = element_height.max(glyph.metrics.size[1] * element.scale);
                }
            }

            // Store bounding box for ray testing
            self.bounding_boxes.push(BoundingBox {
                rect: Rect {
                    x: cursor_x,
                    y: cursor_y - element_height,
                    width: element_width,
                    height: element_height,
                },
                element_id: element.element_id,
            });

            // Second pass: generate vertices
            cursor_x = element.position[0]; // Reset cursor
            for c in element.text.chars() {
                if let Some(glyph) = atlas.get_glyph(c) {
                    self.add_glyph_quad(
                        cursor_x,
                        cursor_y,
                        element.scale,
                        glyph,
                        element.color,
                        element.element_id,
                    );
                    cursor_x += glyph.metrics.advance * element.scale;
                }
            }
        }
    }

    fn add_glyph_quad(
        &mut self,
        x: f32,
        y: f32,
        scale: f32,
        glyph: &GlyphInfo,
        color: [f32; 4],
        element_id: u32,
    ) {
        let base_index = self.vertices.len() as u32;
        let metrics = &glyph.metrics;
        let uv = &glyph.uv_rect;

        // Calculate vertex positions
        let x0 = x + metrics.bearing[0] * scale;
        let y0 = y - (metrics.size[1] - metrics.bearing[1]) * scale;
        let x1 = x0 + metrics.size[0] * scale;
        let y1 = y0 + metrics.size[1] * scale;

        // Add vertices
        self.vertices.extend_from_slice(&[
            TextVertex {
                position: [x0, y0],
                tex_coords: [uv.x, uv.y],
                color,
                element_id,
            },
            TextVertex {
                position: [x1, y0],
                tex_coords: [uv.x + uv.width, uv.y],
                color,
                element_id,
            },
            TextVertex {
                position: [x1, y1],
                tex_coords: [uv.x + uv.width, uv.y + uv.height],
                color,
                element_id,
            },
            TextVertex {
                position: [x0, y1],
                tex_coords: [uv.x, uv.y + uv.height],
                color,
                element_id,
            },
        ]);

        // Add indices for two triangles
        self.indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    pub fn vertices(&self) -> &[TextVertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn bounding_boxes(&self) -> &[BoundingBox] {
        &self.bounding_boxes
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }
}
