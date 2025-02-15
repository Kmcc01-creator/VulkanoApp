#!/bin/bash
set -e

# Initialize a new project
echo "Initializing project..."
vtrack init demo_project
cd demo_project

# Create source file with blocking code
cat > src/lib.rs << 'EOF'
pub fn read_file(path: &str) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

pub fn process_data(data: &str) -> Vec<String> {
    std::thread::sleep(std::time::Duration::from_secs(1));
    data.lines().map(String::from).collect()
}
EOF

# Analyze the code
echo -e "\nAnalyzing code for patterns..."
vtrack analyze src/lib.rs

# Create a script to fix issues
cat > fix.script << 'EOF'
// Convert blocking operations to async
analyze src/lib.rs {
    detect blocking_io
    detect thread_blocking
}

// Generate async versions
generate AsyncFile struct {
    field path: String
}

apply_pattern async_wrapper AsyncFile {
    target = "read_file"
    error_handling = "true"
}

// Generate metrics collection
generate Metrics struct {
    field operations: AtomicU64
    field errors: AtomicU64
}

apply_pattern metrics Metrics {
    prometheus = "true"
}
EOF

# Run the script
echo -e "\nRunning transformation script..."
vtrack script fix.script

# Generate a new API client
echo -e "\nGenerating API client..."
vtrack generate \
    --kind struct \
    --name ApiClient \
    --output src/client.rs \
    --options "fields=base_url,timeout" \
    --options "async=true" \
    --options "error_handling=true"

# Apply builder pattern
cat > builder.script << 'EOF'
apply_pattern builder ApiClient {
    validation = "true"
    doc = "true"
}
EOF

echo -e "\nApplying builder pattern..."
vtrack script builder.script

# Final analysis
echo -e "\nFinal code analysis..."
vtrack analyze src/ --against src/lib.rs.bak

echo -e "\nDemo complete!"
EOF

# Make the script executable
chmod +x run_demo.sh

# Create a README for the demo
cat > README.md << 'EOF'
# Version Tracking Tools Demo

This demo shows how to use the `vtrack` CLI tool to:

1. Initialize a project
2. Analyze code for patterns
3. Transform blocking code to async
4. Generate new code structures
5. Apply common patterns
6. Track changes

## Running the Demo

```bash
./run_demo.sh
```

## Expected Output

The demo will:

1. Create a sample project
2. Find blocking operations
3. Generate async alternatives
4. Add metrics collection
5. Create an API client with builder pattern
6. Show the changes made

## Step-by-Step Explanation

1. **Initialization**
   - Creates a new project with default configuration

2. **Code Analysis**
   - Detects blocking I/O operations
   - Identifies thread-blocking code

3. **Code Generation**
   - Creates async wrapper types
   - Adds metrics collection

4. **Pattern Application**
   - Applies builder pattern
   - Adds documentation
   - Implements error handling

5. **Change Tracking**
   - Shows breaking changes
   - Lists improvements made

## Next Steps

After running the demo, you can:

1. Examine the generated code
2. Try different patterns
3. Customize the configuration
4. Add your own transformations
EOF