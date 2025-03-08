use criterion::{criterion_group, criterion_main, Criterion, black_box};
use gibberish_or_not::{is_gibberish, GibberishDetector, Sensitivity, default_model_path};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const ENGLISH_TEXT_SAMPLES: &[&str] = &[
    // Short sentences
    "Hello world!",
    "This is a simple test.",
    "How are you doing today?",
    
    // Medium sentences
    "The quick brown fox jumps over the lazy dog and runs into the forest.",
    "In computer programming, a string is traditionally a sequence of characters.",
    
    // Long paragraphs
    "Machine learning is a field of inquiry devoted to understanding and building methods \
    that learn, that is, methods that leverage data to improve performance on some set of \
    tasks. It is seen as a part of artificial intelligence.",
    
    // Code snippets
    "fn main() { println!(\"Hello, world!\"); }",
    "public class HelloWorld { public static void main(String[] args) { } }",
    
    // Special characters
    "Hello! How are you? I'm doing great! 😊 #coding @home",
    "Test with numbers: 123, symbols: @#$%, and emoji: 🚀✨",
];

const GIBBERISH_TEXT_SAMPLES: &[&str] = &[
    // Random character strings
    "asdfkjasldkfj",
    "qwertyuiopzxcv",
    
    // Word salad
    "blue dancing quickly elephant mountain",
    "paper taste cloud running sideways",
    
    // Pronounceable non-words
    "zorkle mipnot frandish",
    "quibblenox zentrap",
    
    // Base64-like strings
    "SGVsbG8gd29ybGQh",
    "QWxhZGRpbjpvcGVuIHNlc2FtZQ==",
    
    // Mixed gibberish
    "h3ll0_w0rld_1337",
    "xX_l33t_Xx_n0sc0p3",
];

fn get_text_hash(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

pub fn basic_detection_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic_detection");
    
    // Benchmark English text samples
    for text in ENGLISH_TEXT_SAMPLES {
        let hash = get_text_hash(text);
        group.bench_function(
            format!("english_{}_hash{}", text.len(), hash),
            |b| b.iter(|| is_gibberish(black_box(text), Sensitivity::Medium))
        );
    }

    // Benchmark gibberish text samples
    for text in GIBBERISH_TEXT_SAMPLES {
        let hash = get_text_hash(text);
        group.bench_function(
            format!("gibberish_{}_hash{}", text.len(), hash),
            |b| b.iter(|| is_gibberish(black_box(text), Sensitivity::Medium))
        );
    }

    // Test different sensitivity levels
    let sample_text = "This is a test text for sensitivity levels.";
    let hash = get_text_hash(sample_text);
    for sensitivity in [Sensitivity::Low, Sensitivity::Medium, Sensitivity::High] {
        group.bench_function(
            format!("sensitivity_{:?}_hash{}", sensitivity, hash),
            |b| b.iter(|| is_gibberish(black_box(sample_text), sensitivity))
        );
    }

    group.finish();
}

pub fn bert_detection_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("bert_detection");
    
    // Load the BERT model for enhanced detection
    let model_path = default_model_path();
    let detector = GibberishDetector::with_model(model_path);

    if !detector.has_enhanced_detection() {
        println!("BERT model not available, skipping enhanced detection benchmarks");
        return;
    }

    // Benchmark English text samples
    for text in ENGLISH_TEXT_SAMPLES {
        let hash = get_text_hash(text);
        group.bench_function(
            format!("english_{}_hash{}", text.len(), hash),
            |b| b.iter(|| detector.is_gibberish(black_box(text), Sensitivity::Medium))
        );
    }

    // Benchmark gibberish text samples
    for text in GIBBERISH_TEXT_SAMPLES {
        let hash = get_text_hash(text);
        group.bench_function(
            format!("gibberish_{}_hash{}", text.len(), hash),
            |b| b.iter(|| detector.is_gibberish(black_box(text), Sensitivity::Medium))
        );
    }

    // Test different sensitivity levels
    let sample_text = "This is a test text for sensitivity levels.";
    let hash = get_text_hash(sample_text);
    for sensitivity in [Sensitivity::Low, Sensitivity::Medium, Sensitivity::High] {
        group.bench_function(
            format!("sensitivity_{:?}_hash{}", sensitivity, hash),
            |b| b.iter(|| detector.is_gibberish(black_box(sample_text), sensitivity))
        );
    }

    // Benchmark batch processing (multiple texts at once)
    let batch_texts = vec![
        "This is a normal English sentence.",
        "asdfkjasldkfj",
        "The quick brown fox jumps over the lazy dog.",
        "qwertyuiopzxcv",
    ];
    group.bench_function("batch_processing", |b| {
        b.iter(|| {
            for text in &batch_texts {
                black_box(detector.is_gibberish(text, Sensitivity::Medium));
            }
        })
    });

    group.finish();
}

criterion_group!(benches, basic_detection_benchmark, bert_detection_benchmark);
criterion_main!(benches); 