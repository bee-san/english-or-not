use std::path::PathBuf;
use gibberish_or_not::{
    download_model_with_progress_bar, 
    default_model_path, 
    model_exists, 
    ModelError,
    check_token_status,
    TokenStatus,
};
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

fn main() {
    let args = Args::parse();
    
    // Get model path
    let model_path = args.model_path
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(default_model_path);
    
    // Check if model already exists
    if model_exists(&model_path) && !args.force {
        println!("Model already exists at: {}", model_path.display());
        println!("Use --force to download again.");
        return;
    }
    
    // Check token status
    match check_token_status(&model_path) {
        TokenStatus::Required => {
            eprintln!("HuggingFace token not found. Please set the HUGGING_FACE_HUB_TOKEN environment variable.");
            eprintln!("1. Create an account at https://huggingface.co");
            eprintln!("2. Generate a token at https://huggingface.co/settings/tokens");
            eprintln!("3. Set the token: export HUGGING_FACE_HUB_TOKEN=your_token_here");
            return;
        }
        TokenStatus::NotRequired => {
            println!("Model exists, no token required.");
        }
        TokenStatus::Available => {
            println!("HuggingFace token found, proceeding with download...");
        }
    }
    
    println!("Downloading model to: {}", model_path.display());
    match download_model_with_progress_bar(&model_path) {
        Ok(_) => {
            println!("\nModel downloaded successfully!");
            println!("You can now use enhanced detection with:");
            println!("  cargo run --bin enhanced_detection");
        }
        Err(e) => {
            eprintln!("\nError downloading model: {}", e);
            eprintln!("If this is an authentication error, make sure your HuggingFace token is correct.");
        }
    }
}