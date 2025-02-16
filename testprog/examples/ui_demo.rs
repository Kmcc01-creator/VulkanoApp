use glam::{Vec2, Vec3, Vec4};
use testprog::core::engine::Engine;
use testprog::ui::widget::{Button, Window, WorldSpaceElement};
use testprog::ui::{Element, Layout, Padding, Style};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::new("UI Demo")?;

    // Create a top-layer window with buttons
    let mut window = Element::new()
        .with_id("main-window")
        .with_widget(Window::new("Controls"))
        .with_style(
            Style::new()
                .with_background_color(Vec4::new(0.2, 0.2, 0.2, 0.9))
                .with_border_color(Vec4::new(0.3, 0.3, 0.3, 1.0))
                .with_border_width(1.0)
                .with_text_color(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .with_title_bar_color(Vec4::new(0.3, 0.3, 0.3, 1.0)),
        )
        .with_layout(
            Layout::new()
                .with_margin(Vec2::new(10.0, 10.0))
                .with_padding(Vec2::new(5.0, 5.0))
                .with_min_size(Vec2::new(200.0, 150.0)),
        );

    // Add buttons to the window
    window.add_child(
        Element::new()
            .with_id("button1")
            .with_widget(Button::new("Create Object"))
            .with_style(Style::new().with_padding(Padding::uniform(5.0)))
            .on_click(|| println!("Creating object...")),
    );

    window.add_child(
        Element::new()
            .with_id("button2")
            .with_widget(Button::new("Delete Object"))
            .with_style(Style::new().with_padding(Padding::uniform(5.0)))
            .on_click(|| println!("Deleting object...")),
    );

    // Create in-game UI elements
    let object_label = Element::new()
        .with_id("object-label")
        .with_widget(WorldSpaceElement::new(
            Button::new("3D Object"),
            Vec3::new(0.0, 1.0, 0.0), // Position above an object
        ))
        .with_style(
            Style::new()
                .with_background_color(Vec4::new(0.2, 0.2, 0.2, 0.8))
                .with_text_color(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .with_corner_radius(5.0),
        )
        .on_hover(|hover| println!("Object hover: {}", hover));

    // Add UI elements to the engine
    engine.add_ui_element(window);
    engine.add_ui_element(object_label);

    // Main loop
    while engine.running() {
        engine.update()?;
        engine.render()?;
    }

    Ok(())
}
