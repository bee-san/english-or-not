use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::collections::HashSet;
use std::path::Path;
use encoding_rs::{UTF_8, UTF_16LE, UTF_16BE};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input-file-or-dir> <output-rs-file>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = &args[2];
    let mut seen_passwords = HashSet::new();
    
    // Process either a single file or directory
    if input_path.is_dir() {
        for entry in std::fs::read_dir(input_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && !path.file_name().unwrap().to_string_lossy().starts_with('.') {
                println!("Processing file: {}", path.display());
                process_file(&path, &mut seen_passwords)?;
            }
        }
    } else if input_path.is_file() {
        println!("Processing single file: {}", input_path.display());
        process_file(input_path, &mut seen_passwords)?;
    } else {
        eprintln!("Error: {} is neither a file nor a directory", input_path.display());
        std::process::exit(1);
    }

    println!("Total unique passwords found: {}", seen_passwords.len());

    // Create output Rust file after processing all inputs
    let mut output = File::create(output_path)?;
    writeln!(output, "use phf::phf_set;\n")?;
    writeln!(output, "pub static PASSWORDS: phf::Set<&'static str> = phf_set! {{")?;

    for password in seen_passwords.iter() {
        writeln!(output, "    \"{}\",", password)?;
    }

    writeln!(output, "}};")?;
    Ok(())
}

fn process_file(path: &Path, seen_passwords: &mut HashSet<String>) -> io::Result<()> {
    let bytes = std::fs::read(path)?;
    
    let (encoding, bom_length) = if bytes.starts_with(&[0xFF, 0xFE]) {
        (UTF_16LE, 2)
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        (UTF_16BE, 2)
    } else if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        (UTF_8, 3)
    } else {
        (UTF_8, 0)
    };

    let (text, _, _) = encoding.decode(&bytes[bom_length..]);
    
    for line in text.lines() {
        let password = line.trim();
        if !password.is_empty() && !password.contains(char::is_whitespace) {
            seen_passwords.insert(password.to_owned()); // Don't convert case, keep original
        }
    }
    
    Ok(())
}