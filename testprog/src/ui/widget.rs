use super::{Element, Layout, Rect, Style};
use crate::core::Error;
use crate::graphics::Graphics;

pub trait Widget {
    fn update(&mut self);
    fn render(&self, rect: Rect, style: &Style, graphics: &mut Graphics) -> Result<(), Error>;
}

pub struct WidgetBuilder {
    layout: Layout,
    style: Style,
}

impl WidgetBuilder {
    pub fn new() -> Self {
        Self {
            layout: Layout::default(),
            style: Style::default(),
        }
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn build<W: Widget + 'static>(self, widget: W) -> Element {
        let mut element = Element::with_widget(widget);
        element.set_layout(self.layout);
        element.set_style(self.style);
        element
    }
}

impl Default for WidgetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Basic widgets

pub struct Button {
    text: String,
    clicked: bool,
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            clicked: false,
        }
    }

    pub fn was_clicked(&self) -> bool {
        self.clicked
    }
}

impl Widget for Button {
    fn update(&mut self) {
        // TODO: Implement button interaction
        self.clicked = false;
    }

    fn render(&self, rect: Rect, style: &Style, graphics: &mut Graphics) -> Result<(), Error> {
        // TODO: Implement button rendering
        Ok(())
    }
}
