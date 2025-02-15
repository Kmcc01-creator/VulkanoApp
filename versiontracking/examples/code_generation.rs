use versiontracking::{code_generator::CodeTransformation, VersionTracker};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = VersionTracker::new(None)?;

    // Create a transformation that will generate a new crate
    let mut transformation = CodeTransformation::new();

    // Include specific items we want to transform
    transformation.include_trait("Asset");
    transformation.include_struct("AssetHandle");
    transformation.include_struct("ModelAsset");
    transformation.include_struct("TextureAsset");
    transformation.include_impl("AssetHandle");

    // Generate a new crate from the source
    println!("Analyzing source code...");
    tracker.generate_crate(
        "src/resource",          // Source directory containing the code to analyze
        "generated/resource_v2", // Target directory for the generated code
        transformation,
    )?;

    println!("Generated new crate in generated/resource_v2/");

    // Example of how to analyze just the changes between old and new versions
    if let Ok(changes) =
        tracker.compare_files("src/resource/asset.rs", "generated/resource_v2/lib.rs")
    {
        println!("\nBreaking changes between versions:");
        for change in changes {
            println!("- {:?}", change);
        }
    }

    Ok(())
}
