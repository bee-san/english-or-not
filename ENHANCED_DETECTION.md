# Enhanced Gibberish Detection

This document explains how to use the enhanced gibberish detection feature that integrates with the HuggingFace Inference API.

## Overview

The enhanced detection uses a transformer-based model trained specifically for gibberish detection. It provides more accurate results for borderline cases that might confuse the basic detection algorithm.

The model classifies text into four categories:
1. **Noise** - Complete gibberish (considered gibberish)
2. **Word Salad** - Random but real words (considered gibberish)
3. **Mild gibberish** - Text with grammatical errors (considered not gibberish)
4. **Clean** - Valid English text (considered not gibberish)

## Setup

### Download the Model

Run the download_model tool:

```bash
cargo run --bin download_model
```
This will download the model files (approximately 400-500MB) to your cache directory.

The download may take several minutes depending on your internet connection.

### Test the Enhanced Detection

Run the enhanced_detection tool:

```bash
cargo run --bin enhanced_detection
```

This will let you interactively test the enhanced detection.

## Using in Your Code

### Basic Usage

```rust
use gibberish_or_not::{GibberishDetector, Sensitivity, default_model_path};

// Create detector with model
let detector = GibberishDetector::with_model(default_model_path());

// Check if text is gibberish
let result = detector.is_gibberish("Test text", Sensitivity::Medium);
println!("Is gibberish: {}", result);
```

### Fallback to Basic Detection

If the model is not available or you want to use basic detection:

```rust
use gibberish_or_not::{GibberishDetector, Sensitivity};

// Create detector without model
let detector = GibberishDetector::new();

// Check if text is gibberish (uses only basic algorithm)
let result = detector.is_gibberish("Test text", Sensitivity::Medium);
println!("Is gibberish: {}", result);
```

### Checking if Enhanced Detection is Available

```rust
use gibberish_or_not::{GibberishDetector, default_model_path};

let detector = GibberishDetector::with_model(default_model_path());
if detector.has_enhanced_detection() {
    println!("Enhanced detection is available");
} else {
    println!("Using basic detection only");
}
```

## How It Works

1. The basic algorithm runs first (dictionary and n-gram based checks)
2. If the text is classified as gibberish by the basic algorithm, it returns immediately
3. If the text passes the basic check, the transformer model is used for enhanced detection
4. The model returns a classification, which is converted to a binary result
5. If the model fails, it falls back to the basic algorithm's result

## Notes

- The model is bundled with the library and runs locally
- The model is optimized for English text
- The model requires approximately 400-500MB of disk space