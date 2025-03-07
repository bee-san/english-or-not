use std::env;
use std::io::{self, BufRead};
use gibberish_or_not::{GibberishDetector, Sensitivity, default_model_path, model_exists};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Check for help flag
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_usage();
        return;
    }
    
    // Parse arguments
    let mut sensitivity = Sensitivity::Medium;
    let mut model_path = default_model_path();
    let mut force_basic = false;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--sensitivity" if i + 1 < args.len() => {
                sensitivity = match args[i + 1].to_lowercase().as_str() {
                    "high" => Sensitivity::High,
                    "low" => Sensitivity::Low,
                    _ => Sensitivity::Medium,
                };
                i += 2;
            },
            "-p" | "--path" if i + 1 < args.len() => {
                model_path = std::path::PathBuf::from(&args[i + 1]);
                i += 2;
            },
            "-b" | "--basic" => {
                force_basic = true;
                i += 1;
            },
            _ => {
                i += 1;
            }
        }
    }
    
    // Create detector based on model availability and user preference
    let detector = if !force_basic && model_exists(&model_path) {
        println!("Using enhanced detection with model at: {}", model_path.display());
        GibberishDetector::with_model(&model_path)
    } else {
        if force_basic {
            println!("Using basic detection only (forced by --basic flag).");
        } else {
            println!("Model not found at: {}", model_path.display());
            println!("Using basic detection only.");
            println!("Run 'download_model [PATH]' to enable enhanced detection.");
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
                println!("'{}' is {}", text, if result { "GIBBERISH" } else { "NOT GIBBERISH" });
            }
            Ok(_) => continue,
            Err(_) => break,
        }
    }
}

fn print_usage() {
    println!("Enhanced Detection - Test the enhanced gibberish detection");
    println!("\nUsage:");
    println!("  enhanced_detection [OPTIONS]");
    println!("\nOptions:");
    println!("  -h, --help                 Show this help message");
    println!("  -s, --sensitivity LEVEL    Set sensitivity level (high, medium, low)");
    println!("                             Default: medium");
    println!("  -p, --path PATH            Specify custom model path");
    println!("                             Default: {}", default_model_path().display());
    println!("  -b, --basic                Force basic detection only (ignore model)");
}