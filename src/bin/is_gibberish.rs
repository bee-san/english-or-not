use std::env;
use gibberish_or_not::{is_gibberish, Sensitivity};

fn print_usage(program: &str) {
    eprintln!("Usage: {} <text> [sensitivity]", program);
    eprintln!("  sensitivity: low (default), medium, high");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 || args.len() > 3 {
        print_usage(&args[0]);
    }

    let text = &args[1];
    
    // Parse sensitivity level, defaulting to low
    let sensitivity = if args.len() == 3 {
        match args[2].to_lowercase().as_str() {
            "low" => Sensitivity::Low,
            "medium" => Sensitivity::Medium,
            "high" => Sensitivity::High,
            _ => {
                eprintln!("Invalid sensitivity level. Must be one of: low, medium, high");
                print_usage(&args[0]);
                unreachable!();
            }
        }
    } else {
        Sensitivity::Low
    };

    if is_gibberish(text, sensitivity) {
        println!("This text appears to be gibberish");
    } else {
        println!("This text appears to be valid English");
    }
}
