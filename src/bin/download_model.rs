use std::path::{Path, PathBuf};
use std::io::{self, Write};
use gibberish_or_not::{download_model, default_model_path, model_exists};
use clap::{Parser, CommandFactory};

#[derive(Parser, Debug)]
#[command(author, version, about = "Download enhanced gibberish detection model")]
struct Args {
    /// Path where to save the model files (default: system cache directory)
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Force download even if files exist
    #[arg(short, long)]
    force: bool,
}

fn main() {
    let args = Args::parse();
    
    // Get model path
    let path = args.path.unwrap_or_else(default_model_path);
    
    println!("Downloading model to: {}", path.display());

    // Check if model already exists
    if !args.force && model_exists(&path) {
        println!("Model already exists at: {}", path.display());
        println!("Use --force to download again");
        return;
    }
    
    // Download with progress reporting
    match download_model(&path, |progress| {
        print!("\rDownload progress: {:.1}%", progress * 100.0);
        let _ = io::stdout().flush(); // Ignore flush errors
    }) {
        Ok(_) => {
            println!("\nModel downloaded successfully to: {}", path.display());
            print_usage_help(&path);
        },
        Err(e) => {
            eprintln!("\nError downloading model: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_usage_help(path: &Path) {
    println!("\nYou can now use enhanced gibberish detection with:");
    println!("cargo run --bin enhanced_detection");
    println!("\nOr in your code:");
    println!("let detector = GibberishDetector::with_model(\"{}\");", path.display());
    println!("let result = detector.is_gibberish(\"text\", Sensitivity::Medium);");
}