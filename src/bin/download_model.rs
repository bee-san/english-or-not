use std::path::PathBuf;
use std::env;
use std::io::{self, Write};
use gibberish_or_not::{download_model_with_progress_bar, default_model_path, model_exists};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Check for help flag
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_usage();
        return;
    }
    
    // Get model path from command line or use default
    let path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        default_model_path()
    };
    
    println!("Downloading model to: {}", path.display());
    
    // Check if model already exists
    if model_exists(&path) {
        println!("Model already exists at: {}", path.display());
        println!("Do you want to re-download it? (y/N)");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Download cancelled.");
            return;
        }
    }
    
    // Download the model
    match download_model_with_progress_bar(&path) {
        Ok(_) => {
            println!("\nYou can now use enhanced gibberish detection with:");
            println!("cargo run --bin enhanced_detection");
            println!("\nOr in your code:");
            println!("let detector = GibberishDetector::with_model(\"{}\");", path.display());
            println!("let result = detector.is_gibberish(\"text\", Sensitivity::Medium);");
        },
        Err(e) => {
            eprintln!("\nError downloading model: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Download Model - Downloads the enhanced gibberish detection model");
    println!("\nUsage:");
    println!("  download_model [PATH]");
    println!("\nArguments:");
    println!("  PATH  Optional path where the model should be downloaded");
    println!("        If not provided, the model will be downloaded to:");
    println!("        {}", default_model_path().display());
}