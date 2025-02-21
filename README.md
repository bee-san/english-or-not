# English Text Detector

A Rust library that determines whether a given text is likely to be English using multiple analysis techniques. The library is particularly effective at distinguishing between actual English text and random character sequences, gibberish, or non-English text.

## How It Works

The library uses a comprehensive approach combining six different analysis methods:

1. **Character N-grams Analysis**
   - Bigrams (2-character sequences)
   - Trigrams (3-character sequences)
   - Quadgrams (4-character sequences)
   Each n-gram type is weighted differently in the final score.

2. **Dictionary Check with Perfect Hash Table**
   - Uses a comprehensive list of 370,000+ English words from dwyl/english-words
   - Implements a perfect hash table using a generated match statement
   - Words are compiled into the binary for zero runtime loading overhead
   - Provides O(1) lookup time for word validation
   - Handles both individual words and multi-word phrases
   - Includes common technical terms and proper nouns
   - Memory efficient with no runtime allocations

### Scoring Weightings

The scoring system uses carefully chosen weights based on empirical testing and linguistic research:

1. **N-grams (60% total weight)**
   - Bigrams: 15% - Common two-letter combinations
   - Trigrams: 20% - Frequent three-letter patterns
   - Quadgrams: 25% - Important four-letter sequences

2. **Dictionary Check (15%)**
   - Validates actual word usage
   - Helps distinguish between real words and random letter combinations

3. **Letter Frequency (15%)**
   - Compares text to standard English letter distribution
   - Helps detect texts with unusual letter patterns

4. **Vowel-Consonant Ratio (10%)**
   - Maintains natural balance between vowels and consonants
   - Helps identify texts with unnatural sound patterns

These weights were determined through:
- Analysis of large English text corpora
- Testing against various types of input (valid English, gibberish, mixed content)
- Iterative refinement to maximize accuracy
- Consideration of computational efficiency

3. **Letter Frequency Analysis**
   - Compares letter frequencies with standard English letter distribution
   - Uses empirical frequency data for all 26 letters

4. **Vowel-Consonant Ratio**
   - Analyzes the balance between vowels and consonants
   - Compares against typical English ratio (around 0.5)

The process involves:
1. Text preprocessing (lowercase conversion, removal of non-English characters)
2. Multiple parallel analyses (n-grams, words, letters, ratios)
3. Weighted scoring system combining all metrics
4. Threshold-based final classification

## Usage

Add this to your `Cargo.toml`:

```toml
[
dependencies
]
english_detector = "0.1.0"
```

Basic usage example:

```rust
use english_detector::is_english;

fn main() {
    let text = "The quick brown fox jumps over the lazy dog";
    if is_english(text) {
        println!("This is English text!");
    } else {
        println!("This is not English text.");
    }
}
```

## Examples

The library handles various types of text effectively:

### Regular English Text
```rust
// Standard English sentences
assert!(is_english("The quick brown fox jumps over the lazy dog"));
assert!(is_english("This is a simple English sentence"));

// Technical content
assert!(is_english("The HTTP protocol uses TCP/IP"));
assert!(is_english("README.md contains documentation"));
```

### Non-English and Gibberish
```rust
// Random characters
assert!(!is_english("xkcd vwpq mntb zzzz"));
assert!(!is_english("zzzzz xxxxx qqqqq"));

// Numbers and symbols
assert!(!is_english("12345 67890"));
assert!(!is_english("!@#$%^&*()"));
```

### Mixed Content
```rust
// Text with numbers (valid)
assert!(is_english("I have 5 apples and 3 oranges"));
assert!(is_english("Send email to contact@example.com"));

// Leetspeak (invalid)
assert!(!is_english("H3ll0 W0rld!!!111"));
```

### Edge Cases
```rust
// Short text
assert!(is_english("The cat"));
assert!(is_english("I am"));
assert!(!is_english("xy"));

// Repetitive patterns
assert!(!is_english("aaaaaaaaaaaaaaa"));
assert!(!is_english("thththththththth"));
```

## Technical Details

### Scoring System

The library uses a weighted scoring system that combines multiple metrics:

```rust
let combined_score = 
    0.15 * bigram_score +      // 15% weight
    0.20 * trigram_score +     // 20% weight
    0.25 * quadgram_score +    // 25% weight
    0.15 * letter_freq_score + // 15% weight
    0.10 * vowel_ratio_score + // 10% weight
    0.15 * word_score;         // 15% weight
```

A text is considered English if the combined score is ≥ 0.20 (20%).

### Pattern Sets

The library includes:
- 40 most common English words
- 24 common quadgrams
- 50 common trigrams
- 40 common bigrams
- Complete letter frequency distribution
- Vowel-consonant ratio analysis

### Performance Characteristics

- Fast processing for short to medium texts
- Linear time complexity O(n) where n is text length
- Memory usage proportional to text length
- Thread-safe and `Send + Sync`

## Future Improvements and TODO List

### Accuracy Improvements
- [ ] Add support for common abbreviations and contractions
- [ ] Implement position-aware n-gram scoring
- [ ] Add word-level n-grams analysis
- [ ] Consider implementing machine learning classification
- [ ] Add handling for common misspellings

### Feature Enhancements
- [ ] Return confidence scores (0.0 to 1.0)
- [ ] Add language variant detection (US/UK English)
- [ ] Support for analyzing text streams
- [ ] Configurable scoring weights
- [ ] Add CLI interface for easy usage
- [ ] Create web API endpoint

### Performance Optimizations
- [ ] Implement parallel processing for long texts
- [ ] Add result caching for repeated checks
- [ ] Optimize memory usage for large texts
- [ ] Add SIMD optimizations
- [ ] Explore bloom filters for faster word lookups

### Developer Experience
- [ ] Add comprehensive API documentation
- [ ] Provide Jupyter notebook examples
- [ ] Create benchmarking suite
- [ ] Add fuzzing tests
- [ ] Improve error handling and logging
- [ ] Add more detailed test cases

### CI/CD Improvements
- [ ] Add code coverage reporting
- [ ] Implement automated benchmarking
- [ ] Add security scanning
- [ ] Set up automated dependency updates

### Documentation
- [ ] Add usage examples for different scenarios
- [ ] Create architecture overview
- [ ] Add performance characteristics
- [ ] Document the scoring algorithm in detail

## Contributing

Contributions are welcome! Areas that would particularly benefit from community input:
- Additional test cases for edge cases
- Performance optimizations
- New analysis metrics
- Documentation improvements

## License

This project is licensed under the MIT License - see the LICENSE file for details..
