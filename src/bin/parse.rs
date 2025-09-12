use anyhow::Result;
use clap::Parser;
use std::path::Path;

use semtools::{
    DoclingBackend, DoclingConfig, DoclingServeBackend, DoclingServeConfig, LlamaParseBackend,
    LlamaParseConfig,
};

#[derive(Parser, Debug)]
#[command(version, about = "A CLI tool for parsing documents using various backends", long_about = None)]
struct Args {
    /// Path to the config file. Defaults to ~/.parse_config.json
    #[clap(short = 'c', long)]
    parse_config: Option<String>,

    /// The backend type to use for parsing. Defaults to `llama-parse`
    #[clap(short, long, default_value = "llama-parse")]
    backend: String,

    /// Files to parse
    #[clap(required = true)]
    files: Vec<String>,

    /// Verbose output while parsing
    #[clap(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Get config file path
    let config_path = args.parse_config.unwrap_or_else(|| {
        let config_name = match args.backend.as_str() {
            "docling" => ".docling_config.json",
            "docling-serve" => ".docling_serve_config.json",
            _ => ".parse_config.json",
        };
        dirs::home_dir()
            .unwrap()
            .join(config_name)
            .to_string_lossy()
            .to_string()
    });

    // Validate that files exist
    for file in &args.files {
        if !Path::new(file).exists() {
            eprintln!("Warning: File does not exist: {file}");
        }
    }

    // Create backend and process files
    match args.backend.as_str() {
        "llama-parse" => {
            let config = LlamaParseConfig::from_config_file(&config_path)?;
            let backend = LlamaParseBackend::new(config, args.verbose)?;
            let results = backend.parse(args.files).await?;

            // Output the paths to parsed files, one per line
            for result_path in results {
                println!("{result_path}");
            }
        }
        "docling" => {
            let config = DoclingConfig::from_config_file(&config_path)?;
            let backend = DoclingBackend::new(config, args.verbose)?;
            let results = backend.parse(args.files).await?;

            // Output the paths to parsed files, one per line
            for result_path in results {
                println!("{result_path}");
            }
        }
        "docling-serve" => {
            let config = DoclingServeConfig::from_config_file(&config_path)?;
            let backend = DoclingServeBackend::new(config, args.verbose)?;
            let results = backend.parse(args.files).await?;

            // Output the paths to parsed files, one per line
            for result_path in results {
                println!("{result_path}");
            }
        }
        _ => {
            eprintln!(
                "Error: Unknown backend '{}'. Supported backends: llama-parse, docling, docling-serve",
                args.backend
            );
            std::process::exit(1);
        }
    }

    Ok(())
}
