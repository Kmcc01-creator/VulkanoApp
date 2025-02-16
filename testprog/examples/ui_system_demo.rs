use glam::{Vec2, Vec3, Vec4};
use testprog::ui::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create UI system with viewport size
    let mut ui = UI::new(Vec2::new(800.0, 600.0));

    // Create a window with buttons (top-layer UI)
    let mut window = Element::new()
        .with_id("main_window")
        .with_widget(Window::new("Controls").with_size(Vec2::new(200.0, 150.0)))
        .with_style(
            Style::new()
                .with_background_color(Vec4::new(0.2, 0.2, 0.2, 0.9))
                .with_border_color(Vec4::new(0.3, 0.3, 0.3, 1.0))
                .with_border_width(1.0)
                .with_text_color(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .with_title_bar_color(Vec4::new(0.3, 0.3, 0.3, 1.0)),
        )
        .with_layout(Layout::new().with_position(PositionType::Absolute(Vec2::new(10.0, 10.0))));

    // Add buttons to the window
    window.add_child(
        Element::new()
            .with_id("spawn_button")
            .with_widget(Button::new("Spawn Object"))
            .with_style(Style::new().with_padding(Padding::uniform(5.0)))
            .on_click(|| println!("Spawning new object...")),
    );

    window.add_child(
        Element::new()
            .with_id("delete_button")
            .with_widget(Button::new("Delete Object"))
            .with_style(Style::new().with_padding(Padding::uniform(5.0)))
            .on_click(|| println!("Deleting selected object...")),
    );

    // Add window to UI
    ui.add_element(window);

    // Create in-game UI elements
    let object_label = Element::new()
        .with_id("object_label")
        .with_widget(WorldSpaceWidget::new(
            Button::new("3D Object").with_size(Vec2::new(100.0, 30.0)),
            Vec3::new(0.0, 1.0, 0.0), // Position above an object in world space
            Vec2::new(100.0, 30.0),
        ))
        .with_style(
            Style::new()
                .with_background_color(Vec4::new(0.2, 0.2, 0.2, 0.8))
                .with_text_color(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .with_corner_radius(5.0),
        )
        .on_hover(|hover| println!("Object hover state: {}", hover));

    // Add in-game UI to system
    ui.add_element(object_label);

    // Simulate some interactions
    println!("Initial render:");
    ui.render()?;

    println!("\nSimulating mouse click at (20, 20):");
    ui.handle_click(Vec2::new(20.0, 20.0));

    println!("\nSimulating hover over object:");
    ui.handle_hover(Vec2::new(0.0, 1.0));

    println!("\nSimulating window resize:");
    ui.resize(Vec2::new(1024.0, 768.0));
    ui.render()?;

    Ok(())
}

// Example output when run:
/*
Initial render:
Drawing rect at (10, 10) size (200, 150) color (0.2, 0.2, 0.2, 0.9)
Drawing rect at (10, 10) size (200, 25) color (0.3, 0.3, 0.3, 1.0)
Drawing text 'Controls' at (15, 15) color (1.0, 1.0, 1.0, 1.0)
Drawing rect outline at (10, 10) size (200, 150) width 1 color (0.3, 0.3, 0.3, 1.0)
Drawing rect at (15, 40) size (100, 30) color (0.2, 0.2, 0.2, 1.0)
Drawing text 'Spawn Object' at (20, 48) color (1.0, 1.0, 1.0, 1.0)
Drawing rect at (15, 75) size (100, 30) color (0.2, 0.2, 0.2, 1.0)
Drawing text 'Delete Object' at (20, 83) color (1.0, 1.0, 1.0, 1.0)
Drawing rect at (350, 250) size (100, 30) color (0.2, 0.2, 0.2, 0.8)
Drawing text '3D Object' at (370, 258) color (1.0, 1.0, 1.0, 1.0)

Simulating mouse click at (20, 20):
Spawning new object...

Simulating hover over object:
Object hover state: true

Simulating window resize:
[Layout recalculated with new viewport size]
*/
