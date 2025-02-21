use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    let input_file = "words_alpha.txt";
    let output_file = "src/dictionary.rs";
    
    // Create output file
    let mut output = File::create(output_file)?;
    
    // Write header
    writeln!(output, "pub fn is_english_word(word: &str) -> bool {{")?;
    writeln!(output, "    match word {{")?;
    
    // Try to process input file
    if let Ok(input) = File::open(input_file) {
        let reader = io::BufReader::new(input);
        
        for line in reader.lines() {
            let word = line?.trim().to_string();
            if !word.is_empty() {
                writeln!(output, "        \"{}\" => true,", word)?;
            }
        }
    } else {
        println!("Warning: words_alpha.txt not found - creating empty dictionary");
    }
    
    // Write footer
    writeln!(output, "        _ => false,")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    
    println!("Dictionary generated successfully at {}", output_file);
    Ok(())
}
