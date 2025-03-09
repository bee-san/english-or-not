use clap::{Parser};
use gibberish_or_not::{default_model_path, model_exists, GibberishDetector, Sensitivity};
use std::io::{self, BufRead};

#[derive(Parser, Debug)]
#[command(author, version, about = "Test the enhanced gibberish detection")]
struct Args {
    /// Path to model directory (default: system cache directory)
    #[arg(short, long)]
    model_path: Option<String>,

    /// Sensitivity level (low, medium, high)
    #[arg(short, long, default_value = "medium")]
    sensitivity: String,

    /// Force basic detection even if model is available
    #[arg(short, long)]
    basic: bool,
}

fn main() {
    let args = Args::parse();

    // Determine sensitivity level
    let sensitivity = match args.sensitivity.to_lowercase().as_str() {
        "high" => Sensitivity::High,
        "low" => Sensitivity::Low,
        _ => Sensitivity::Medium,
    };

    // Get model path and check if model exists
    let model_path = args
        .model_path
        .map(|p| p.into())
        .unwrap_or_else(default_model_path);

    // Create detector based on model availability and user preference
    let detector = if !args.basic && model_exists(&model_path) {
        println!(
            "Using enhanced detection with model at: {}",
            model_path.display()
        );
        GibberishDetector::with_model(model_path)
    } else {
        if args.basic {
            println!("Using basic detection (--basic flag provided)");
        } else {
            println!("Model not found at: {}", model_path.display());
            println!("Using basic detection. Run 'cargo run --bin download_model' to enable enhanced detection.");
        }
        GibberishDetector::new()
    };

    println!("Enter text to check (Ctrl+D to exit):");
    println!("Using sensitivity: {:?}", sensitivity);

    // Read lines from stdin
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(text) if !text.is_empty() => {
                let result = detector.is_gibberish(&text, sensitivity);
                println!(
                    "'{}' is {}",
                    text,
                    if result { "GIBBERISH" } else { "NOT GIBBERISH" }
                );
            }
            Ok(_) => continue,
            Err(_) => break,
        }
    }
}
