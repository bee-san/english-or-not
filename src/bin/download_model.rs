use std::path::PathBuf;
use gibberish_or_not::{download_model_with_progress_bar, default_model_path, model_exists};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Download the enhanced gibberish detection model")]
struct Args {
    /// Path to download model to (default: system cache directory)
    #[arg(short, long)]
    model_path: Option<String>,
    
    /// Force download even if model already exists
    #[arg(short, long)]
    force: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Get model path
    let model_path = args.model_path
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(default_model_path);
    
    // Check if model already exists
    if model_exists(&model_path) && !args.force {
        println!("Model already exists at: {}", model_path.display());
        println!("Use --force to download again.");
        return Ok(());
    }
    
    println!("Downloading model to: {}", model_path.display());
    download_model_with_progress_bar(&model_path)?;
    
    println!("Model downloaded successfully!");
    println!("You can now use enhanced detection with:");
    println!("  cargo run --bin enhanced_detection");
    
    Ok(())
}