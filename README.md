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
gibberish-or-not = "1.0.0"
```

## 🎯 Examples

```rust
use gibberish_or_not::{is_gibberish, Sensitivity};

// Password Detection
assert!(is_password("123456"));  // Detects common passwords

// Valid English
assert!(!is_gibberish("The quick brown fox jumps over the lazy dog", Sensitivity::Medium));
assert!(!is_gibberish("Technical terms like TCP/IP and README.md work too", Sensitivity::Medium));

// Gibberish
assert!(is_gibberish("asdf jkl qwerty", Sensitivity::Medium));
assert!(is_gibberish("xkcd vwpq mntb", Sensitivity::Medium));
```

## 🔬 How It Works

Our advanced detection algorithm uses three main components:

### 1. 📚 Dictionary Analysis
- **370,000+ English words** compiled into the binary
- Perfect hash table for O(1) lookups
- Zero runtime loading overhead
- Includes technical terms and proper nouns

### 2. 🧮 N-gram Analysis
- **Trigrams** (3-letter sequences)
  - Needs >15% match for single-word texts (Bletchley)
  - Needs >10% match for no-word texts (TextThatLooksLikeThisWhichCouldTripItUpOtherwise)
- **Quadgrams** (4-letter sequences)
  - Needs >10% match for single-word texts
  - Needs >5% match for no-word texts (TextThatLooksLikeThisWhichCouldTripItUpOtherwise)
- Trained on massive English text corpus

### 3. 🎯 Smart Classification
- Text with 2+ English words → Valid English
- Text with 1 English word → Must pass n-gram thresholds
- Text with no English words → Must pass lower n-gram thresholds
- Short text (<10 chars) → Dictionary check only (not enough data for n-grams)

## 🎚️ Sensitivity Levels

The library provides three sensitivity levels to fine-tune gibberish detection:

### Low Sensitivity
- Most strict classification
- Requires very high confidence to classify text as English
- Best for detecting texts that appear English-like but are actually gibberish
- Thresholds:
  - 2+ English words: Needs >20% trigram/quadgram match
  - 1 English word: Needs >25% trigram/quadgram match
  - No English words: Always classified as gibberish

### Medium Sensitivity (Default)
- Balanced approach for general use
- Combines dictionary and n-gram analysis
- Default mode suitable for most applications
- Thresholds:
  - 2+ English words: Automatically classified as English
  - 1 English word: Needs >15% trigram or >10% quadgram match
  - No English words: Needs >10% trigram or >5% quadgram match

### High Sensitivity
- Most lenient classification
- Favors classifying text as English
- Best when input is mostly gibberish and any English-like patterns are significant
- Thresholds:
  - Any English word: Automatically classified as English
  - No English words: Needs >5% trigram or >3% quadgram match

```rust
use gibberish_or_not::{is_gibberish, Sensitivity};

// Example text with one English word "iron"
let text = "Rcl maocr otmwi lit dnoen oehc 13 iron seah.";

// Different results based on sensitivity
assert!(is_gibberish(text, Sensitivity::Low));    // Classified as gibberish
assert!(!is_gibberish(text, Sensitivity::Medium)); // Classified as English
assert!(!is_gibberish(text, Sensitivity::High));   // Classified as English
```

## 🔑 Password Detection

The library includes functionality to detect common passwords:

### Detection Method
- Uses a comprehensive list of over 10,000 common passwords
- Performs exact matching against known passwords
- Supports multiple encodings (UTF-8, UTF-16)
- Zero runtime loading overhead using perfect hash table

### Example Usage

```rust
use gibberish_or_not::is_password;

// Check if a string is a common password
assert!(is_password("123456"));        // True - very common password
assert!(is_password("P@ssw0rd"));      // True - common password
assert!(!is_password("_a_super_unique_password_skibidi_ohio_rizz")); // False - not in common password list
```

## � Contributing

Contributions are welcome! Here's how you can help:

- 🐛 Report bugs and request features
- 📝 Improve documentation
- 🔧 Submit pull requests
- 💡 Share ideas and feedback

## 📜 License

MIT License - see the [LICENSE](LICENSE) file for details.

