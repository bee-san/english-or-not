# Enhanced Gibberish Detection

This document explains how to use the enhanced gibberish detection library.

## Overview

The enhanced detection uses a transformer-based model trained specifically for gibberish detection. It provides more accurate results for borderline cases that might confuse the basic detection algorithm.

The model classifies text into four categories:
1. **Noise** - Complete gibberish (considered gibberish)
2. **Word Salad** - Random but real words (considered gibberish)
3. **Mild gibberish** - Text with grammatical errors (considered not gibberish)
4. **Clean** - Valid English text (considered not gibberish)

## Usage

### Managing the Model

First, you'll need to download the model. The library provides functions to manage this:

```rust
use gibberish_or_not::{download_model_with_progress_bar, default_model_path, model_exists};
use std::error::Error;

fn setup() -> Result<(), Box<dyn Error>> {
    let path = default_model_path();
    
    if !model_exists(&path) {
        download_model_with_progress_bar(&path)?;
    }
    
    Ok(())
}
```

For custom progress reporting:

```rust
use gibberish_or_not::{download_model, ModelError};
use std::path::Path;

fn download_with_custom_progress() -> Result<(), ModelError> {
    let path = Path::new("./model_dir");
    
    download_model(path, |progress| {
        // Handle progress updates (0.0 to 1.0)
        // Example: update a progress bar in your UI
    })
}
```

### Using the Detector

#### With Enhanced Detection

```rust
use gibberish_or_not::{GibberishDetector, Sensitivity, default_model_path};

// Create detector with model
let detector = GibberishDetector::with_model(default_model_path());
let result = detector.is_gibberish("Test text", Sensitivity::Medium);
```

#### With Basic Detection

```rust
use gibberish_or_not::{GibberishDetector, Sensitivity};

// Create detector without model
let detector = GibberishDetector::new();
let result = detector.is_gibberish("Test text", Sensitivity::Medium);
```

#### Checking Enhanced Detection Availability

```rust
use gibberish_or_not::{GibberishDetector, default_model_path};

let detector = GibberishDetector::with_model(default_model_path());
if detector.has_enhanced_detection() {
    // Use enhanced detection
} else {
    // Fall back to basic detection
}
```

### Example Integration

```rust
use gibberish_or_not::{GibberishDetector, Sensitivity, default_model_path, model_exists, download_model_with_progress_bar};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = default_model_path();
    
    // Download model if needed
    if !model_exists(&path) {
        download_model_with_progress_bar(&path)?;
    }
    
    // Create detector
    let detector = GibberishDetector::with_model(&path);
    
    // Process text
    let text = "Sample text to analyze";
    let is_gibberish = detector.is_gibberish(text, Sensitivity::Medium);
    
    Ok(())
}
```

## How It Works

1. The basic algorithm runs first (dictionary and n-gram based checks)
2. If the text is classified as gibberish by the basic algorithm, it returns immediately
3. If the text passes the basic check, the transformer model is used for enhanced detection
4. The model returns a classification, which is converted to a binary result
5. If the model fails, it falls back to the basic algorithm's result

## Notes

- The model is downloaded on demand and runs locally
- The library focuses on functionality over CLI interaction
- The model is optimized for English text
- The model requires approximately 400-500MB of disk space
- Enhanced detection is optional and will only be used if the model is available