use testprog::core::Engine;

fn main() {
    // Create and initialize the engine
    let mut engine = Engine::new().expect("Failed to create engine");

    // Enable input debugging to see input state in console
    engine.enable_input_debug();

    println!("Engine initialized. Move mouse and click to test input system.");
    println!("Close window to exit.");

    // Run the engine
    if let Err(e) = engine.run() {
        eprintln!("Engine error: {}", e);
    }
}
