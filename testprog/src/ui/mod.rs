mod element;
mod layout;
mod mock_renderer;
mod style;
mod widget;

pub use element::{Element, Rect};
pub use layout::{Layout, PositionType};
pub use mock_renderer::MockRenderer;
pub use style::{Padding, Style};
pub use widget::{Button, Widget, Window, WorldSpaceWidget};

use crate::core::error::Error;
use glam::Vec2;

pub struct UI {
    renderer: MockRenderer,
    root: Element,
    viewport_size: Vec2,
}

impl UI {
    pub fn new(viewport_size: Vec2) -> Self {
        Self {
            renderer: MockRenderer::new(viewport_size),
            viewport_size,
            root: Element::new()
                .with_id("root")
                .with_layout(Layout::new().with_min_size(viewport_size)),
        }
    }

    pub fn add_element(&mut self, element: Element) {
        self.root.add_child(element);
    }

    pub fn render(&mut self) -> Result<(), Error> {
        // Recalculate layout
        let viewport_bounds = Rect::new(Vec2::ZERO, self.viewport_size);
        self.root.layout(viewport_bounds);

        // Render UI tree
        self.root.render(&mut self.renderer)
    }

    pub fn update(&mut self) {
        self.root.update();
    }

    pub fn handle_click(&mut self, position: Vec2) {
        if self.root.computed_bounds.contains(position) {
            self.root.handle_click();
        }
    }

    pub fn handle_hover(&mut self, position: Vec2) {
        if self.root.computed_bounds.contains(position) {
            self.root.handle_hover(true);
        } else {
            self.root.handle_hover(false);
        }
    }

    pub fn handle_key(&mut self, key: char) {
        self.root.handle_key(key);
    }

    pub fn resize(&mut self, new_size: Vec2) {
        self.viewport_size = new_size;
        let viewport_bounds = Rect::new(Vec2::ZERO, new_size);
        self.root.layout(viewport_bounds);
    }
}

// Re-export everything needed for UI creation
pub mod prelude {
    pub use super::{Button, Window, WorldSpaceWidget};
    pub use super::{Element, Layout, Padding, PositionType, Style, UI};
    pub use glam::{Vec2, Vec3, Vec4};
}
