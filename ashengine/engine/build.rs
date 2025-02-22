use std::path::Path;
use std::process::Command;

fn main() {
    // Create spv directory if it doesn't exist
    let spv_dir = Path::new("shaders/spv");
    std::fs::create_dir_all(spv_dir).unwrap();

    // Compile vertex shader
    println!("cargo:rerun-if-changed=shaders/text.vert");
    let status = Command::new("glslc")
        .args(&["shaders/text.vert", "-o", "shaders/spv/text.vert.spv"])
        .status()
        .expect("Failed to execute glslc command");

    if !status.success() {
        panic!("Failed to compile vertex shader");
    }

    // Compile fragment shader
    println!("cargo:rerun-if-changed=shaders/text.frag");
    let status = Command::new("glslc")
        .args(&["shaders/text.frag", "-o", "shaders/spv/text.frag.spv"])
        .status()
        .expect("Failed to execute glslc command");

    if !status.success() {
        panic!("Failed to compile fragment shader");
    }
}
