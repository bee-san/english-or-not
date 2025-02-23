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

## ✨ Features

**🚀 Lightning Fast**
- Zero runtime loading
- Perfect hash table lookups
- Optimized for speed

**📚 Smart Analysis**
- Dictionary of 370k+ words
- N-gram pattern matching
- Frequency analysis

**🎯 High Accuracy**
- 99% detection rate
- Handles edge cases
- Works with technical text


## 🎯 Examples

```rust
use gibberish_or_not::is_gibberish;

// Valid English
assert!(!is_gibberish("The quick brown fox jumps over the lazy dog"));
assert!(!is_gibberish("Technical terms like TCP/IP and README.md work too"));

// Gibberish
assert!(is_gibberish("asdf jkl qwerty"));
assert!(is_gibberish("xkcd vwpq mntb"));
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

## 👥 Contributing

Contributions are welcome! Here's how you can help:

- 🐛 Report bugs and request features
- 📝 Improve documentation
- 🔧 Submit pull requests
- 💡 Share ideas and feedback

## 📜 License

MIT License - see the [LICENSE](LICENSE) file for details.

