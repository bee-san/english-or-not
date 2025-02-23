use std::env;
use std::fs::File;
use std::io::{self, Write};
use encoding_rs::{UTF_8, UTF_16LE, UTF_16BE};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input-file> <output-rs-file>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    
    // Read file as raw bytes for encoding detection
    let bytes = std::fs::read(input_path)?;
    
    // Detect encoding from BOM (Byte Order Mark)
    let (encoding, bom_length) = if bytes.starts_with(&[0xFF, 0xFE]) {
        (UTF_16LE, 2)
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        (UTF_16BE, 2)
    } else if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        (UTF_8, 3)
    } else {
        (UTF_8, 0)
    };

    // Decode the file using detected encoding
    let (text, _, _) = encoding.decode(&bytes[bom_length..]);
    
    // Create output Rust file
    let mut output = File::create(output_path)?;
    writeln!(output, "use phf::phf_set;\n")?;
    writeln!(output, "pub static ENGLISH_WORDS: phf::Set<&'static str> = phf_set! {{")?;

    // Process lines - only trim whitespace don't modify content
    for line in text.lines() {
        let word = line.trim(); // Trims \r\n and whitespace but preserves inner characters
        if !word.is_empty() {
            writeln!(output, "    \"{}\",", word)?;
        }
    }

    writeln!(output, "}};")?;
    Ok(())
}
