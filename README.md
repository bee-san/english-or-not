<div align="center">

# 🔍 Gibberish Detection Tool

**Instantly detect if text is English or nonsense with 99% accuracy**

[![Crates.io](https://img.shields.io/crates/v/gibberish-or-not.svg)](https://crates.io/crates/gibberish-or-not)
[![Documentation](https://docs.rs/gibberish-or-not/badge.svg)](https://docs.rs/gibberish-or-not)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[Documentation](https://docs.rs/gibberish-or-not) |
[Examples](#examples) |
[Contributing](#contributing)

</div>

## ⚡ Quick Install

```bash
# As a CLI tool
cargo install gibberish-or-not

# As a library in Cargo.toml
gibberish-or-not = "4.0.3"
```

## 🎯 Examples

```rust
use gibberish_or_not::{is_gibberish, is_password, Sensitivity};

// Password Detection
assert!(is_password("123456"));  // Detects common passwords

// Valid English
assert!(!is_gibberish("The quick brown fox jumps over the lazy dog", Sensitivity::Medium));
assert!(!is_gibberish("Hello, world!", Sensitivity::Medium));

// Gibberish
assert!(is_gibberish("asdf jkl qwerty", Sensitivity::Medium));
assert!(is_gibberish("xkcd vwpq mntb", Sensitivity::Medium));
assert!(is_gibberish("println!({});", Sensitivity::Medium)); // Code snippets are classified as gibberish
```

## 🔬 How It Works

Our advanced detection algorithm uses multiple components:

### 1. 📚 Dictionary Analysis
- **370,000+ English words** compiled into the binary
- Perfect hash table for O(1) lookups
- Zero runtime loading overhead
- Includes technical terms and proper nouns

### 2. 🧮 N-gram Analysis
- **Trigrams** (3-letter sequences)
- **Quadgrams** (4-letter sequences)
- Trained on massive English text corpus
- Weighted scoring system

### 3. 🎯 Smart Classification
- Composite scoring system combining:
  - English word ratio (40% weight)
  - Character transition probability (25% weight)
  - Trigram analysis (15% weight)
  - Quadgram analysis (10% weight)
  - Vowel-consonant ratio (10% weight)
- Length-based threshold adjustment
- Special case handling for:
  - Very short text (<10 chars)
  - Non-printable characters
  - Code snippets
  - URLs and technical content

## 🎚️ Sensitivity Levels

The library provides three sensitivity levels:

### High Sensitivity
- Most lenient classification
- Easily accepts text as English
- Best for minimizing false positives
- Use when: You want to catch anything remotely English-like

### Medium Sensitivity (Default)
- Balanced approach
- Suitable for general text classification
- Reliable for most use cases
- Use when: You want general-purpose gibberish detection

### Low Sensitivity
- Most strict classification
- Requires strong evidence of English
- Best for security applications
- Use when: False positives are costly

## 🔑 Password Detection

Built-in detection of common passwords:

```rust
use gibberish_or_not::is_password;

assert!(is_password("123456"));     // Common password
assert!(is_password("password"));   // Common password
assert!(!is_password("unique_and_secure_passphrase")); // Not in common list
```

## 🎯 Special Cases

The library handles various special cases:

- Code snippets are classified as gibberish
- URLs in text are preserved for analysis
- Technical terms and abbreviations are recognized
- Mixed-language content is supported
- ASCII art is detected as gibberish
- Common internet text patterns are recognized

## 🤝 Contributing

Contributions are welcome! Please feel free to:

- Report bugs and request features
- Improve documentation
- Submit pull requests
- Add test cases

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

