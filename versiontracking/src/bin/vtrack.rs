use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use versiontracking::{
    scripting::{Interpreter, ScriptConfig},
    Analyzer, CodeGenerator, GenerationTarget, GeneratorConfig,
};

#[derive(Parser)]
#[command(name = "vtrack")]
#[command(about = "Version tracking and code generation tools", long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = ".version-tracking.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze code for patterns and breaking changes
    Analyze {
        /// Files or directories to analyze
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Check for breaking changes against this version
        #[arg(short, long)]
        against: Option<PathBuf>,
    },

    /// Generate code from patterns
    Generate {
        /// Generation target type (struct, function, trait)
        #[arg(short, long)]
        kind: String,

        /// Target name
        #[arg(short, long)]
        name: String,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,

        /// Additional fields or options (key=value)
        #[arg(short, long)]
        options: Vec<String>,
    },

    /// Run a script file
    Script {
        /// Script file to run
        #[arg(required = true)]
        file: PathBuf,
    },

    /// Initialize a new configuration
    Init {
        /// Directory to initialize
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Load configuration
    let config = if cli.config.exists() {
        ScriptConfig::from_file(&cli.config)?
    } else {
        ScriptConfig::default_config()
    };

    match cli.command {
        Commands::Analyze { paths, against } => {
            analyze_paths(paths, against, &config).await?;
        }

        Commands::Generate {
            kind,
            name,
            output,
            options,
        } => {
            generate_code(kind, name, output, options, &config).await?;
        }

        Commands::Script { file } => {
            run_script(file, &config).await?;
        }

        Commands::Init { path } => {
            initialize_config(path)?;
        }
    }

    Ok(())
}

async fn analyze_paths(
    paths: Vec<PathBuf>,
    against: Option<PathBuf>,
    config: &ScriptConfig,
) -> Result<()> {
    let analyzer = Analyzer::new();

    for path in paths {
        println!("Analyzing {}", path.display());

        let content = std::fs::read_to_string(&path)?;
        let source = syn::parse_str(&content)?;
        let analysis = analyzer.analyze(&source);

        // Print findings
        println!("\nPatterns found:");
        for pattern in &analysis.patterns {
            println!("  - {}: {}", pattern.pattern, pattern.context);
        }

        // Compare if requested
        if let Some(ref old_path) = against {
            let old_content = std::fs::read_to_string(old_path)?;
            let old_source = syn::parse_str(&old_content)?;

            let changes = analyzer.compare_versions(&old_source, &source);
            if !changes.is_empty() {
                println!("\nBreaking changes:");
                for change in changes {
                    println!("  - {}", change);
                }
            }
        }
    }

    Ok(())
}

async fn generate_code(
    kind: String,
    name: String,
    output: PathBuf,
    options: Vec<String>,
    config: &ScriptConfig,
) -> Result<()> {
    // Parse options
    let mut opts = std::collections::HashMap::new();
    for opt in options {
        if let Some((k, v)) = opt.split_once('=') {
            opts.insert(k.to_string(), v.to_string());
        }
    }

    // Create generation target
    let target = match kind.as_str() {
        "struct" => GenerationTarget::Struct {
            name: name.clone(),
            fields: opts
                .get("fields")
                .map(|f| f.split(',').map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        },
        "function" => GenerationTarget::Function {
            name: name.clone(),
            is_async: opts.get("async").map_or(false, |v| v == "true"),
            is_unsafe: opts.get("unsafe").map_or(false, |v| v == "true"),
        },
        "trait" => GenerationTarget::Trait {
            name: name.clone(),
            methods: opts
                .get("methods")
                .map(|f| f.split(',').map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        },
        _ => anyhow::bail!("Unknown generation kind: {}", kind),
    };

    // Configure generator
    let generator_config = GeneratorConfig {
        documentation: config.generation.documentation,
        ..Default::default()
    };

    // Generate code
    let generator = CodeGenerator::new(generator_config);
    let code = generator.generate(target).await?;

    // Write output
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, code.to_string())?;

    Ok(())
}

async fn run_script(file: PathBuf, config: &ScriptConfig) -> Result<()> {
    let content = std::fs::read_to_string(file)?;
    let mut interpreter = Interpreter::new();
    interpreter.execute_script(&content).await
}

fn initialize_config(path: PathBuf) -> Result<()> {
    let config = ScriptConfig::default_config();
    let config_path = path.join(".version-tracking.toml");

    if config_path.exists() {
        println!(
            "Configuration file already exists at {}",
            config_path.display()
        );
        return Ok(());
    }

    config.save_to_file(config_path)?;
    println!("Initialized configuration file");
    Ok(())
}
