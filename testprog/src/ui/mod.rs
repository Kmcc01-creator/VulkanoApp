mod element;
mod layout;
mod style;
mod widget;

pub use element::Element;
pub use layout::Layout;
pub use style::Style;
pub use widget::{Widget, WidgetBuilder};

use crate::core::error::Error;
use crate::graphics::Graphics;
use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
}

pub struct UIContext {
    root: Element,
    style_sheet: Style,
}

impl UIContext {
    pub fn new() -> Self {
        Self {
            root: Element::new(),
            style_sheet: Style::default(),
        }
    }

    pub fn add_widget<W: Widget + 'static>(&mut self, widget: W) -> &mut Element {
        let element = Element::with_widget(widget);
        self.root.add_child(element)
    }

    pub fn update(&mut self) {
        self.root.update();
        self.layout();
    }

    pub fn render(&self, graphics: &mut Graphics) -> Result<(), Error> {
        self.root.render(graphics)
    }

    fn layout(&mut self) {
        let viewport = Rect {
            position: Vec2::ZERO,
            size: Vec2::new(800.0, 600.0), // Default viewport size
        };
        self.root.layout(viewport, &self.style_sheet);
    }
}

impl Default for UIContext {
    fn default() -> Self {
        Self::new()
    }
}
