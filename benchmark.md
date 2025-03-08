# Benchmarking Gibberish Detection

This document outlines a benchmarking strategy for the `gibberish-or-not` library, focusing on comparing the performance of basic detection versus enhanced detection (using the BERT model).

## Benchmarking Goals

* **Compare execution time:** Measure the time taken for both basic and enhanced detection on various input types.
* **Analyze sensitivity impact:** Evaluate how different sensitivity levels (Low, Medium, High) affect performance.
* **Assess input length influence:** Determine how text length impacts processing time.
* **Identify bottlenecks:** Pinpoint any performance bottlenecks in either detection method.

## Benchmarking Setup

### 1. Data Preparation

* **English Text:**
    * Use a diverse set of English texts, including:
        * Short sentences (1-10 words)
        * Medium sentences (11-30 words)
        * Long paragraphs (50+ words)
        * Code snippets
        * Text with special characters and punctuation
        * Real-world examples from different domains (e.g., news articles, technical documentation, literature)
    * Vary text length within each category to assess length impact.
* **Gibberish Text:**
    * Generate various types of gibberish, including:
        * Random character strings
        * Word salad (random English words)
        * Pronounceable non-words
        * Base64 encoded strings
        * Adversarial examples designed to fool the basic algorithm
    * Vary gibberish length similar to English text.

### 2. Benchmarking Framework

Use the `criterion` crate for robust benchmarking.  It provides statistical analysis and handles variations in execution time.

```rust
use criterion::{criterion_group, criterion_main, Criterion, black_box};
use gibberish_or_not::{is_gibberish, GibberishDetector, Sensitivity, default_model_path};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("gibberish_detection");
    
    // Load or download the BERT model for enhanced detection
    let model_path = default_model_path();
    let detector = GibberishDetector::with_model(model_path);

    for text in ENGLISH_TEXT_SAMPLES {
        group.bench_with_input(format!("basic_english_{}", text.len()), text, |b, &text| {
            b.iter(|| is_gibberish(black_box(text), Sensitivity::Medium)); // Basic detection
        });
        if detector.has_enhanced_detection() {
            group.bench_with_input(format!("enhanced_english_{}", text.len()), text, |b, &text| {
                b.iter(|| detector.is_gibberish(black_box(text), Sensitivity::Medium)); // Enhanced detection
            });
        }
    }

    for text in GIBBERISH_TEXT_SAMPLES {
        group.bench_with_input(format!("basic_gibberish_{}", text.len()), text, |b, &text| {
            b.iter(|| is_gibberish(black_box(text), Sensitivity::Medium)); // Basic detection
        });
        if detector.has_enhanced_detection() {
            group.bench_with_input(format!("enhanced_gibberish_{}", text.len()), text, |b, &text| {
                b.iter(|| detector.is_gibberish(black_box(text), Sensitivity::Medium)); // Enhanced detection
            });
        }
    }
    
    group.finish();
}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);