use std::env;
use gibberish_or_not::is_gibberish;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <text>", args[0]);
        std::process::exit(1);
    }

    let text = &args[1];
    if is_gibberish(text) {
        println!("This text appears to be gibberish");
    } else {
        println!("This text appears to be valid English");
    }
}
