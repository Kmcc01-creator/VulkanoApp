use glam::Vec2;
use std::collections::HashMap;

use super::layout::Layout;
use super::style::Style;
use crate::core::error::Error;
use crate::graphics::Graphics;

#[derive(Debug, Clone, Default)]
pub struct Element {
    pub id: String,
    pub children: Vec<Element>,
    pub layout: Layout,
    pub style: Style,
    pub computed_bounds: Rect,
    pub events: EventHandlers,
    widget: Option<Box<dyn Widget>>,
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.position.x
            && point.x <= self.position.x + self.size.x
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.y
    }
}

pub trait Widget: std::fmt::Debug {
    fn render(&self, bounds: Rect, graphics: &mut Graphics) -> Result<(), Error>;
    fn update(&mut self);
    fn measure(&self, available_space: Vec2) -> Vec2;
}

#[derive(Debug, Clone, Default)]
pub struct EventHandlers {
    click: Option<Box<dyn Fn() + Send + Sync>>,
    hover: Option<Box<dyn Fn(bool) + Send + Sync>>,
    key: Option<Box<dyn Fn(char) + Send + Sync>>,
    custom: HashMap<String, Box<dyn Fn() + Send + Sync>>,
}

impl Element {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_widget<W: Widget + 'static>(mut self, widget: W) -> Self {
        self.widget = Some(Box::new(widget));
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn add_child(&mut self, child: Element) -> &mut Element {
        self.children.push(child);
        self.children.last_mut().unwrap()
    }

    pub fn render(&self, graphics: &mut Graphics) -> Result<(), Error> {
        // Render self if we have a widget
        if let Some(widget) = &self.widget {
            widget.render(self.computed_bounds.clone(), graphics)?;
        }

        // Render children
        for child in &self.children {
            child.render(graphics)?;
        }

        Ok(())
    }

    pub fn update(&mut self) {
        // Update self if we have a widget
        if let Some(widget) = &mut self.widget {
            widget.update();
        }

        // Update children
        for child in &mut self.children {
            child.update();
        }
    }

    pub fn layout(&mut self, bounds: Rect, parent_style: &Style) {
        // Compute layout based on parent bounds and style
        self.computed_bounds = self.layout.compute(bounds, parent_style);

        // Layout children
        let available_space = Vec2::new(
            self.computed_bounds.size.x - self.style.padding.left - self.style.padding.right,
            self.computed_bounds.size.y - self.style.padding.top - self.style.padding.bottom,
        );

        for child in &mut self.children {
            child.layout(
                Rect::new(
                    self.computed_bounds.position
                        + Vec2::new(self.style.padding.left, self.style.padding.top),
                    available_space,
                ),
                &self.style,
            );
        }
    }

    // Event handling
    pub fn on_click<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.events.click = Some(Box::new(f));
        self
    }

    pub fn on_hover<F: Fn(bool) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.events.hover = Some(Box::new(f));
        self
    }

    pub fn on_key<F: Fn(char) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.events.key = Some(Box::new(f));
        self
    }

    pub fn on_custom<S: Into<String>, F: Fn() + Send + Sync + 'static>(
        mut self,
        event: S,
        f: F,
    ) -> Self {
        self.events.custom.insert(event.into(), Box::new(f));
        self
    }

    pub fn handle_click(&self) {
        if let Some(handler) = &self.events.click {
            handler();
        }
    }

    pub fn handle_hover(&self, is_hover: bool) {
        if let Some(handler) = &self.events.hover {
            handler(is_hover);
        }
    }

    pub fn handle_key(&self, key: char) {
        if let Some(handler) = &self.events.key {
            handler(key);
        }
    }

    pub fn handle_custom(&self, event: &str) {
        if let Some(handler) = self.events.custom.get(event) {
            handler();
        }
    }
}
