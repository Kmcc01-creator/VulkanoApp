use glam::Vec2;
use std::collections::HashMap;
use std::sync::Arc;

use super::layout::Layout;
use super::mock_renderer::MockRenderer;
use super::style::Style;
use super::widget::Widget;
use crate::core::error::Error;

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug)]
pub struct Element {
    pub id: String,
    pub children: Vec<Element>,
    pub layout: Layout,
    pub style: Style,
    pub computed_bounds: Rect,
    events: EventHandlers,
    widget: Option<Arc<dyn Widget>>,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            id: String::new(),
            children: Vec::new(),
            layout: Layout::default(),
            style: Style::default(),
            computed_bounds: Rect::new(Vec2::ZERO, Vec2::ZERO),
            events: EventHandlers::default(),
            widget: None,
        }
    }
}

pub struct EventHandlers {
    click: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    hover: Option<Arc<dyn Fn(bool) + Send + Sync + 'static>>,
    key: Option<Arc<dyn Fn(char) + Send + Sync + 'static>>,
    custom: HashMap<String, Arc<dyn Fn() + Send + Sync + 'static>>,
}

impl Default for EventHandlers {
    fn default() -> Self {
        Self {
            click: None,
            hover: None,
            key: None,
            custom: HashMap::new(),
        }
    }
}

impl std::fmt::Debug for EventHandlers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventHandlers")
            .field("click", &self.click.is_some())
            .field("hover", &self.hover.is_some())
            .field("key", &self.key.is_some())
            .field("custom", &self.custom.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl Element {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_widget(mut self, widget: impl Widget + 'static) -> Self {
        self.widget = Some(Arc::new(widget));
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

    pub fn render(&self, renderer: &mut MockRenderer) -> Result<(), Error> {
        // Render self if we have a widget
        if let Some(widget) = &self.widget {
            widget.render(self.computed_bounds, &self.style, renderer)?;
        }

        // Render children
        for child in &self.children {
            child.render(renderer)?;
        }

        Ok(())
    }

    pub fn update(&mut self) {
        // Update self if we have a widget
        if let Some(widget) = &mut self.widget {
            Arc::get_mut(widget)
                .expect("Cannot mutably update widget with multiple references")
                .update();
        }

        // Update children
        for child in &mut self.children {
            child.update();
        }
    }

    pub fn layout(&mut self, available_space: Rect) {
        // Calculate this element's bounds based on layout constraints
        self.computed_bounds = self.layout.calculate_bounds(available_space, &self.style);

        // Create content area for children (accounting for padding)
        let content_space = self.layout.calculate_content_area(self.computed_bounds);

        // Layout children
        for child in &mut self.children {
            child.layout(content_space);
        }
    }

    // Event handling
    pub fn on_click<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.events.click = Some(Arc::new(f));
        self
    }

    pub fn on_hover<F: Fn(bool) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.events.hover = Some(Arc::new(f));
        self
    }

    pub fn on_key<F: Fn(char) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.events.key = Some(Arc::new(f));
        self
    }

    pub fn on_custom<S: Into<String>, F: Fn() + Send + Sync + 'static>(
        mut self,
        event: S,
        f: F,
    ) -> Self {
        self.events.custom.insert(event.into(), Arc::new(f));
        self
    }

    pub fn handle_click(&self) {
        if let Some(handler) = &self.events.click {
            handler();
        }

        // Propagate to children
        for child in &self.children {
            child.handle_click();
        }
    }

    pub fn handle_hover(&self, is_hover: bool) {
        if let Some(handler) = &self.events.hover {
            handler(is_hover);
        }

        // Propagate to children
        for child in &self.children {
            child.handle_hover(is_hover);
        }
    }

    pub fn handle_key(&self, key: char) {
        if let Some(handler) = &self.events.key {
            handler(key);
        }

        // Propagate to children
        for child in &self.children {
            child.handle_key(key);
        }
    }

    pub fn handle_custom(&self, event: &str) {
        if let Some(handler) = self.events.custom.get(event) {
            handler();
        }

        // Propagate to children
        for child in &self.children {
            child.handle_custom(event);
        }
    }
}
