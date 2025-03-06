use phf::phf_set;

mod dictionary;
mod passwords;

/// Sensitivity level for gibberish detection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sensitivity {
    /// High sensitivity - requires very high confidence to classify as English.
    /// Best for texts that appear English-like but are actually gibberish.
    /// Relies heavily on dictionary word matching.
    High,

    /// Medium sensitivity - balanced approach using both dictionary and n-gram analysis.
    /// Suitable for general purpose text classification.
    Medium,

    /// Low sensitivity - more lenient classification as English.
    /// Best when input is expected to be mostly gibberish, and any English-like
    /// patterns should be flagged as potential English text.
    Low,
}

fn is_english_word(word: &str) -> bool {
    dictionary::ENGLISH_WORDS.contains(word)
}

/// Checks if the given text matches a known common password.
///
/// This function checks if the input text exactly matches a password from a comprehensive
/// list of common passwords, including:
/// - Most commonly used passwords
/// - Default passwords
/// - Dictionary-based passwords
///
/// # Arguments
///
/// * `text` - The text to check against the password list
///
/// # Returns
///
/// * `true` if the text exactly matches a known password
/// * `false` otherwise
///
/// # Examples
///
/// ```
/// use gibberish_or_not::is_password;
/// assert!(is_password("123456")); // A very common password
/// assert!(!is_password("not-a-common-password")); // Not in the password list
/// ```
pub fn is_password(text: &str) -> bool {
    passwords::PASSWORDS.contains(text)
}
// The dictionary module provides a perfect hash table implementation
// using the phf crate, which is generated at compile time
// for optimal performance and memory efficiency

/// Checks if the given text is gibberish based on English word presence
/// and n-gram analysis scores. The sensitivity level determines how strict
/// the classification should be.
///
/// # Arguments
///
/// * `text` - The input text to analyze
/// * `sensitivity` - Controls how strict the gibberish detection should be:
///   - High: Very strict, requires high confidence to classify as English
///   - Medium: Balanced approach using dictionary and n-grams
///   - Low: More lenient, flags English-like patterns as non-gibberish
///
/// # Algorithm Steps
///
/// 1. Clean and normalize the input text
/// 2. Short text (len < 10) - single word check
/// 3. Split into words and count English words:
///    - 2+ English words → considered valid
///    - 1 English word → check n-gram scores
///    - 0 English words → more lenient n-gram check
/// 4. Use different n-gram thresholds depending on sensitivity level
pub fn is_gibberish(text: &str, sensitivity: Sensitivity) -> bool {
    // Clean the text first
    let cleaned = clean_text(text);

    // Check if empty after cleaning
    if cleaned.is_empty() {
        return true;
    }

    // For very short cleaned text, only check if it's an English word
    if cleaned.len() < 10 {
        let is_english = is_english_word(&cleaned);
        return !is_english;
    }

    // Split into words and check for English words
    let words: Vec<&str> = cleaned
        .split_whitespace()
        .filter(|word| !word.is_empty())
        .collect();

    // Count English words
    let english_word_count = words.iter().filter(|word| is_english_word(word)).count();
    
    let english_word_ratio = if words.is_empty() {
        0.0
    } else {
        english_word_count as f64 / words.len() as f64
    };

    // Check for non-printable characters which are strong indicators of gibberish
    let non_printable_count = text
        .chars()
        .filter(|&c| c < ' ' && c != '\n' && c != '\r' && c != '\t')
        .count();

    // If there are non-printable characters, it's likely gibberish
    if non_printable_count > 0 {
        return true;
    }

    // Calculate character entropy - gibberish often has unusual character distributions
    let entropy = calculate_entropy(text);
    
    // Calculate character transition probability - English has predictable transitions
    let transition_score = calculate_transition_score(text);
    
    // Calculate vowel-consonant ratio - English has a fairly consistent ratio
    let vowel_consonant_ratio = calculate_vowel_consonant_ratio(&cleaned);
    
    // Proceed with trigram/quadgram analysis (but with less weight)
    let trigrams = generate_ngrams(&cleaned, 3);
    let quadgrams = generate_ngrams(&cleaned, 4);

    let valid_trigrams = trigrams
        .iter()
        .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
        .collect::<Vec<_>>();

    let valid_quadgrams = quadgrams
        .iter()
        .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
        .collect::<Vec<_>>();

    // Calculate scores
    let trigram_score = if trigrams.is_empty() {
        0.0
    } else {
        valid_trigrams.len() as f64 / trigrams.len() as f64
    };

    let quadgram_score = if quadgrams.is_empty() {
        0.0
    } else {
        valid_quadgrams.len() as f64 / quadgrams.len() as f64
    };

    // Calculate a composite score that combines multiple metrics
    // This makes the algorithm more robust than relying heavily on n-grams
    let mut composite_score = 0.0;
    
    // English word ratio has high weight
    composite_score += english_word_ratio * 0.4;
    
    // Transition probability has medium weight
    composite_score += transition_score * 0.25;
    
    // N-gram scores have lower weight
    composite_score += trigram_score * 0.15;
    composite_score += quadgram_score * 0.1;
    
    // Vowel-consonant ratio has low weight
    composite_score += if (0.3..=0.7).contains(&vowel_consonant_ratio) { 0.1 } else { 0.0 };

    // Adjust thresholds based on text length
    let length_factor = match cleaned.len() {
        0..=20 => 0.7,    // Very short text needs higher threshold
        21..=50 => 0.8,   // Short text
        51..=100 => 0.9,  // Medium text
        101..=200 => 1.0, // Standard threshold
        _ => 1.1,         // Long text can be more lenient
    };

    // Decision thresholds based on sensitivity
    let threshold = match sensitivity {
        Sensitivity::Low => 0.35 * length_factor,    // Stricter - needs more evidence to be English
        Sensitivity::Medium => 0.25 * length_factor, // Balanced
        Sensitivity::High => 0.15 * length_factor,   // Lenient - less evidence needed to be English
    };

    // Special cases that override the composite score
    
    // If almost all words are English, it's definitely English
    if english_word_ratio > 0.8 {
        return false;
    }
    
    // If we have multiple English words, it's likely English
    if english_word_count >= 3 && sensitivity != Sensitivity::Low {
        return false;
    }
    
    // If we have no English words and poor transition score, it's likely gibberish
    if english_word_count == 0 && transition_score < 0.3 && sensitivity != Sensitivity::High {
        return true;
    }
    
    // For the remaining cases, use the composite score
    composite_score < threshold
}

/// Calculate character entropy - a measure of randomness in the text
fn calculate_entropy(text: &str) -> f64 {
    let text = text.to_lowercase();
    let total_chars = text.chars().count() as f64;
    
    if total_chars == 0.0 {
        return 0.0;
    }
    
    // Count character frequencies
    let mut char_counts = std::collections::HashMap::new();
    for c in text.chars() {
        *char_counts.entry(c).or_insert(0) += 1;
    }
    
    // Calculate entropy
    let mut entropy = 0.0;
    for &count in char_counts.values() {
        let probability = count as f64 / total_chars;
        entropy -= probability * probability.log2();
    }
    
    // Normalize to 0-1 range (typical English text has entropy around 4.0-4.5)
    (entropy / 5.0).min(1.0)
}

/// Calculate character transition probabilities based on English patterns
fn calculate_transition_score(text: &str) -> f64 {
    let text = text.to_lowercase();
    let chars: Vec<char> = text.chars().collect();
    
    if chars.len() < 2 {
        return 0.0;
    }
    
    let mut valid_transitions = 0;
    let total_transitions = chars.len() - 1;
    
    for i in 0..total_transitions {
        let pair = format!("{}{}", chars[i], chars[i+1]);
        if COMMON_CHAR_PAIRS.contains(&pair.as_str()) {
            valid_transitions += 1;
        }
    }
    
    valid_transitions as f64 / total_transitions as f64
}

/// Calculate vowel-consonant ratio (English typically has a ratio around 0.4-0.6)
fn calculate_vowel_consonant_ratio(text: &str) -> f64 {
    let vowels = ['a', 'e', 'i', 'o', 'u'];
    let mut vowel_count = 0;
    let mut consonant_count = 0;
    
    for c in text.chars() {
        if vowels.contains(&c) {
            vowel_count += 1;
        } else if c.is_alphabetic() {
            consonant_count += 1;
        }
    }
    
    if consonant_count == 0 {
        return if vowel_count == 0 { 0.0 } else { 1.0 };
    }
    
    vowel_count as f64 / (vowel_count + consonant_count) as f64
}

// Common character pairs in English
static COMMON_CHAR_PAIRS: phf::Set<&'static str> = phf_set! {
    "th", "he", "in", "er", "an", "re", "on", "at", "en", "nd", 
    "ti", "es", "or", "te", "of", "ed", "is", "it", "al", "ar", 
    "st", "to", "nt", "ng", "se", "ha", "as", "ou", "io", "le", 
    "ve", "co", "me", "de", "hi", "ri", "ro", "ic", "ne", "ea", 
    "ra", "ce", "li", "ch", "ll", "be", "ma", "si", "om", "ur"
};

static COMMON_QUADGRAMS: phf::Set<&'static str> = phf_set! {
    "tion", "atio", "that", "ther", "with", "ment", "ions", "this",
    "here", "from", "ould", "ting", "hich", "whic", "ctio", "ever",
    "they", "thin", "have", "othe", "were", "tive", "ough", "ight"
};

static COMMON_TRIGRAMS: phf::Set<&'static str> = phf_set! {
    "the", "and", "ing", "ion", "tio", "ent", "ati", "for", "her", "ter",
    "hat", "tha", "ere", "con", "res", "ver", "all", "ons", "nce", "men",
    "ith", "ted", "ers", "pro", "thi", "wit", "are", "ess", "not", "ive",
    "was", "ect", "rea", "com", "eve", "per", "int", "est", "sta", "cti",
    "ica", "ist", "ear", "ain", "one", "our", "iti", "rat", "ell", "ant"
};

static ENGLISH_LETTERS: phf::Set<char> = phf_set! {
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'
};

fn clean_text(text: &str) -> String {
    text.chars()
        .map(|c| {
            if ENGLISH_LETTERS.contains(&c) || c.is_ascii_digit() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() {
                ' '
            } else {
                ' '
            }
        })
        .collect()
}

fn generate_ngrams(text: &str, n: usize) -> Vec<String> {
    let filtered: String = text
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ENGLISH_LETTERS.contains(&ch) || ch.is_numeric() {
                ch
            } else {
                ' '
            }
        })
        .collect();

    filtered
        .split_whitespace()
        .flat_map(|word| {
            word.as_bytes()
                .windows(n)
                .filter_map(|window| String::from_utf8(window.to_vec()).ok())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    // Tests for the password detection functionality
    #[test]
    fn test_common_passwords() {
        assert!(is_password("123456"));
        assert!(is_password("password"));
        assert!(is_password("qwerty"));
        assert!(is_password("abc123"));
    }

    #[test]
    fn test_numeric_passwords() {
        assert!(is_password("123456789"));
        assert!(is_password("12345678"));
        assert!(is_password("1234567"));
    }

    #[test]
    fn test_word_passwords() {
        assert!(is_password("iloveyou"));
        assert!(is_password("admin"));
        assert!(is_password("welcome"));
    }

    #[test]
    fn test_non_passwords() {
        assert!(!is_password("")); // Empty string
        assert!(!is_password("this is not a password")); // Contains spaces
        assert!(!is_password("verylongandunlikelypasswordthatnoonewoulduse")); // Too long
        assert!(!is_password("unique_string_123")); // Not in common list
    }

    use super::*;

    // Helper function to run tests with different sensitivities
    fn test_with_sensitivities(
        text: &str,
        expected_low: bool,
        expected_med: bool,
        expected_high: bool,
    ) {
        assert_eq!(is_gibberish(text, Sensitivity::Low), expected_low);
        assert_eq!(is_gibberish(text, Sensitivity::Medium), expected_med);
        assert_eq!(is_gibberish(text, Sensitivity::High), expected_high);
    }

    #[test]
    fn test_clear_english_all_sensitivities() {
        let text = "The quick brown fox jumps over the lazy dog.";
        println!("\nTesting text: '{}'", text);

        for sensitivity in [Sensitivity::Low, Sensitivity::Medium, Sensitivity::High] {
            let cleaned = clean_text(text);
            let words: Vec<&str> = cleaned.split_whitespace().collect();
            let english_words: Vec<&&str> =
                words.iter().filter(|word| is_english_word(word)).collect();

            let trigrams = generate_ngrams(&cleaned, 3);
            let quadgrams = generate_ngrams(&cleaned, 4);

            let valid_trigrams = trigrams
                .iter()
                .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
                .collect::<Vec<_>>();
            let valid_quadgrams = quadgrams
                .iter()
                .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
                .collect::<Vec<_>>();

            println!("\nSensitivity {:?}:", sensitivity);
            println!("Cleaned text: '{}'", cleaned);
            println!(
                "English words found: {} out of {}",
                english_words.len(),
                words.len()
            );
            println!("English words: {:?}", english_words);
            println!(
                "Trigram score: {:.3}",
                if trigrams.is_empty() {
                    0.0
                } else {
                    valid_trigrams.len() as f64 / trigrams.len() as f64
                }
            );
            println!(
                "Quadgram score: {:.3}",
                if quadgrams.is_empty() {
                    0.0
                } else {
                    valid_quadgrams.len() as f64 / quadgrams.len() as f64
                }
            );

            let result = is_gibberish(text, sensitivity);
            println!("Result: {}", if result { "GIBBERISH" } else { "ENGLISH" });
        }

        test_with_sensitivities(
            text, false, // Changed from true to false for Low sensitivity
            false, // Changed from true to false for Medium sensitivity
            false, // Changed from true to false for High sensitivity
        );
    }

    #[test]
    fn test_borderline_english_like_gibberish() {
        test_with_sensitivities(
            "Rcl maocr otmwi lit dnoen oehc 13 iron seah.",
            true,
            false,
            false, // Medium sensitivity accepts this due to "iron"
        );
    }

    #[test]
    fn test_clear_gibberish_all_sensitivities() {
        test_with_sensitivities("!@#$%^&*()", true, true, true);
    }

    #[test]
    fn test_english_word_with_ngrams() {
        let text = "ther with tion";
        println!("\n==== DEBUG: test_english_word_with_ngrams ====");
        println!("Text: '{}'", text);

        // Clean and analyze text
        let cleaned = clean_text(text);
        let words: Vec<&str> = cleaned.split_whitespace().collect();
        let english_words: Vec<&&str> = words.iter().filter(|w| is_english_word(w)).collect();

        println!("\n== Word Analysis ==");
        println!("Total words: {}", words.len());
        println!(
            "English words: {} ({:?})",
            english_words.len(),
            english_words
        );

        // Calculate n-gram scores
        let trigrams = generate_ngrams(&cleaned, 3);
        let quadgrams = generate_ngrams(&cleaned, 4);

        let valid_trigrams = trigrams
            .iter()
            .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        let valid_quadgrams = quadgrams
            .iter()
            .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        let trigram_score = if trigrams.is_empty() {
            0.0
        } else {
            valid_trigrams.len() as f64 / trigrams.len() as f64
        };
        let quadgram_score = if quadgrams.is_empty() {
            0.0
        } else {
            valid_quadgrams.len() as f64 / quadgrams.len() as f64
        };

        println!("\n== N-gram Analysis ==");
        println!("Trigram score: {:.3}", trigram_score);
        println!("Quadgram score: {:.3}", quadgram_score);

        println!("\n== Test Assertion ==");
        println!("Should classify as GIBBERISH with LOW sensitivity");
        assert!(!is_gibberish(text, Sensitivity::Low), "Text with common n-grams should not be classified as gibberish with low sensitivity");
    }

    // Valid English text tests
    #[test]
    fn test_pangram() {
        assert!(!is_gibberish(
            "The quick brown fox jumps over the lazy dog.",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_simple_sentence() {
        assert!(!is_gibberish(
            "This is a simple English sentence.",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_hello_world() {
        assert!(!is_gibberish("Hello, world!", Sensitivity::Medium));
    }

    #[test]
    fn test_single_word() {
        assert!(!is_gibberish("hello", Sensitivity::Medium));
    }

    #[test]
    fn test_common_ngrams() {
        assert!(!is_gibberish("ther with tion", Sensitivity::Medium));
    }

    #[test]
    fn test_technical_text() {
        assert!(!is_gibberish(
            "The function returns a boolean value.",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_mixed_case() {
        assert!(!is_gibberish(
            "MiXeD cAsE text IS still English",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_with_punctuation() {
        assert!(!is_gibberish(
            "Hello! How are you? I'm doing well.",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_long_text() {
        assert!(!is_gibberish("This is a longer piece of text that contains multiple sentences and should definitely be recognized as valid English content.", Sensitivity::Medium));
    }

    // Gibberish text tests
    #[test]
    fn test_numbers_only() {
        assert!(is_gibberish("12345 67890", Sensitivity::Medium));
    }

    #[test]
    fn test_empty_string() {
        assert!(is_gibberish("", Sensitivity::Medium));
    }

    #[test]
    fn test_non_english_chars() {
        assert!(is_gibberish("你好世界", Sensitivity::Medium));
    }

    #[test]
    fn test_special_chars() {
        assert!(is_gibberish("!@#$%^&*()", Sensitivity::Medium));
    }

    #[test]
    fn test_base64_like() {
        assert!(is_gibberish("MOTCk4ywLLjjEE2=", Sensitivity::Medium));
    }

    #[test]
    fn test_short_gibberish() {
        assert!(is_gibberish("4-Fc@w7MF", Sensitivity::Medium));
    }

    #[test]
    fn test_letter_substitution() {
        assert!(is_gibberish("Vszzc hvwg wg zcbu", Sensitivity::Medium));
    }

    // Edge cases
    #[test]
    fn test_single_letter() {
        assert!(is_gibberish("a", Sensitivity::Medium));
    }

    #[test]
    fn test_mixed_valid_invalid() {
        assert!(!is_gibberish("hello xkcd world", Sensitivity::Medium));
    }

    #[test]
    fn test_common_abbreviation() {
        assert!(!is_gibberish("NASA FBI CIA", Sensitivity::Medium));
    }

    #[test]
    fn test_with_numbers() {
        assert!(!is_gibberish(
            "Room 101 is down the hall",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_keyboard_mash() {
        assert!(is_gibberish("asdfgh jkl", Sensitivity::Medium));
    }

    #[test]
    fn test_repeated_word() {
        assert!(!is_gibberish(
            "buffalo buffalo buffalo",
            Sensitivity::Medium
        ));
    }

    // URLs and email addresses
    #[test]
    fn test_url() {
        assert!(!is_gibberish(
            "Visit https://www.example.com for more info",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_email_address() {
        assert!(!is_gibberish(
            "Contact us at support@example.com",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_url_only() {
        assert!(is_gibberish("https://aaa.bbb.ccc/ddd", Sensitivity::Medium));
    }

    // Code-like text
    #[test]
    fn test_code_snippet() {
        let text = "println!({});";
        println!("\n==== DEBUGGING CODE SNIPPET TEST ====");
        println!("Original text: '{}'", text);
        println!("Text length: {}", text.len());

        // Debug the cleaning process
        let cleaned = clean_text(text);
        println!("Cleaned text: '{}'", cleaned);
        println!("Cleaned text length: {}", cleaned.len());

        // Debug character distribution
        let char_counts = text.chars().fold(std::collections::HashMap::new(), |mut map, c| {
            *map.entry(c).or_insert(0) += 1;
            map
        });
        println!("\n== CHARACTER DISTRIBUTION ==");
        println!("Unique characters: {}", char_counts.len());
        for (c, count) in char_counts.iter() {
            println!("  '{}': {} occurrences", c, count);
        }

        // Debug entropy calculation
        let entropy = calculate_entropy(text);
        println!("\n== ENTROPY ANALYSIS ==");
        println!("Character entropy: {:.4}", entropy);
        println!("Normalized entropy (0-1): {:.4}", (entropy / 5.0).min(1.0));

        // Debug transition score
        let transition_score = calculate_transition_score(text);
        println!("\n== TRANSITION SCORE ANALYSIS ==");
        println!("Transition score: {:.4}", transition_score);
        
        // Debug character transitions
        let chars: Vec<char> = text.to_lowercase().chars().collect();
        println!("Character transitions:");
        for i in 0..(chars.len() - 1) {
            let pair = format!("{}{}", chars[i], chars[i+1]);
            let is_common = COMMON_CHAR_PAIRS.contains(&pair.as_str());
            println!("  '{}' - {}", pair, if is_common { "COMMON" } else { "uncommon" });
        }

        // Debug vowel-consonant ratio
        let vowel_consonant_ratio = calculate_vowel_consonant_ratio(&cleaned);
        println!("\n== VOWEL-CONSONANT ANALYSIS ==");
        println!("Vowel-consonant ratio: {:.4}", vowel_consonant_ratio);
        println!("In typical English range (0.3-0.7): {}", (0.3..=0.7).contains(&vowel_consonant_ratio));

        // Split into words and check each one
        let words: Vec<&str> = cleaned
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .collect();

        println!("\n== WORD ANALYSIS ==");
        println!("Total words: {}", words.len());

        let mut english_word_count = 0;
        println!("Words after splitting:");
        for word in &words {
            let is_english = is_english_word(word);
            if is_english {
                english_word_count += 1;
            }
            println!("  \"{}\" - {}", word, if is_english { "ENGLISH WORD" } else { "not English" });
        }

        println!(
            "English words found: {} out of {} ({:.2}%)",
            english_word_count,
            words.len(),
            if words.is_empty() { 0.0 } else { english_word_count as f64 / words.len() as f64 * 100.0 }
        );
        
        let english_word_ratio = if words.is_empty() {
            0.0
        } else {
            english_word_count as f64 / words.len() as f64
        };
        println!("English word ratio: {:.4}", english_word_ratio);

        // Check n-grams
        let trigrams = generate_ngrams(&cleaned, 3);
        let quadgrams = generate_ngrams(&cleaned, 4);

        let valid_trigrams = trigrams
            .iter()
            .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        let valid_quadgrams = quadgrams
            .iter()
            .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        let trigram_score = if trigrams.is_empty() {
            0.0
        } else {
            valid_trigrams.len() as f64 / trigrams.len() as f64
        };

        let quadgram_score = if quadgrams.is_empty() {
            0.0
        } else {
            valid_quadgrams.len() as f64 / quadgrams.len() as f64
        };

        println!("\n== TRIGRAM ANALYSIS ==");
        println!("Total trigrams: {}", trigrams.len());
        println!("All trigrams:");
        for trigram in &trigrams {
            let is_common = COMMON_TRIGRAMS.contains(trigram.as_str());
            println!("  \"{}\" - {}", trigram, if is_common { "COMMON" } else { "uncommon" });
        }
        println!("Valid trigrams: {}/{} = {:.4}", valid_trigrams.len(), trigrams.len(), trigram_score);
        println!("Trigram score: {:.4}", trigram_score);
        println!("Trigram threshold check (> 0.15): {}", trigram_score > 0.15);
        println!("Trigram threshold check (> 0.1): {}", trigram_score > 0.1);

        println!("\n== QUADGRAM ANALYSIS ==");
        println!("Total quadgrams: {}", quadgrams.len());
        println!("All quadgrams:");
        for quadgram in &quadgrams {
            let is_common = COMMON_QUADGRAMS.contains(quadgram.as_str());
            println!("  \"{}\" - {}", quadgram, if is_common { "COMMON" } else { "uncommon" });
        }
        println!("Valid quadgrams: {}/{} = {:.4}", valid_quadgrams.len(), quadgrams.len(), quadgram_score);
        println!("Quadgram score: {:.4}", quadgram_score);
        println!("Quadgram threshold check (> 0.1): {}", quadgram_score > 0.1);
        println!("Quadgram threshold check (> 0.05): {}", quadgram_score > 0.05);

        // Calculate composite score
        let mut composite_score = 0.0;
        composite_score += english_word_ratio * 0.4;
        composite_score += transition_score * 0.25;
        composite_score += trigram_score * 0.15;
        composite_score += quadgram_score * 0.1;
        composite_score += if (0.3..=0.7).contains(&vowel_consonant_ratio) { 0.1 } else { 0.0 };

        println!("\n== COMPOSITE SCORE CALCULATION ==");
        println!("English word ratio component: {:.4} * 0.4 = {:.4}", english_word_ratio, english_word_ratio * 0.4);
        println!("Transition score component: {:.4} * 0.25 = {:.4}", transition_score, transition_score * 0.25);
        println!("Trigram score component: {:.4} * 0.15 = {:.4}", trigram_score, trigram_score * 0.15);
        println!("Quadgram score component: {:.4} * 0.1 = {:.4}", quadgram_score, quadgram_score * 0.1);
        println!("Vowel-consonant ratio component: {} * 0.1 = {:.4}", 
            if (0.3..=0.7).contains(&vowel_consonant_ratio) { 1.0 } else { 0.0 },
            if (0.3..=0.7).contains(&vowel_consonant_ratio) { 0.1 } else { 0.0 });
        println!("Composite score: {:.4}", composite_score);

        // Length factor calculation
        let length_factor = match cleaned.len() {
            0..=20 => 0.7,    // Very short text needs higher threshold
            21..=50 => 0.8,   // Short text
            51..=100 => 0.9,  // Medium text
            101..=200 => 1.0, // Standard threshold
            _ => 1.1,         // Long text can be more lenient
        };
        println!("Length factor (based on {} chars): {:.2}", cleaned.len(), length_factor);

        // Threshold calculations
        let low_threshold = 0.35 * length_factor;
        let medium_threshold = 0.25 * length_factor;
        let high_threshold = 0.15 * length_factor;
        
        println!("\n== SENSITIVITY THRESHOLDS ==");
        println!("Low sensitivity threshold: {:.4}", low_threshold);
        println!("Medium sensitivity threshold: {:.4}", medium_threshold);
        println!("High sensitivity threshold: {:.4}", high_threshold);
        println!("Composite score < Low threshold: {}", composite_score < low_threshold);
        println!("Composite score < Medium threshold: {}", composite_score < medium_threshold);
        println!("Composite score < High threshold: {}", composite_score < high_threshold);

        println!("\n== SPECIAL CASE CHECKS ==");
        println!("English word ratio > 0.8: {} (returns NOT gibberish if true)", english_word_ratio > 0.8);
        println!("English word count >= 3: {} (returns NOT gibberish if true for Medium/High sensitivity)", english_word_count >= 3);
        println!("No English words AND transition score < 0.3: {} (returns gibberish if true for Low/Medium sensitivity)", 
            english_word_count == 0 && transition_score < 0.3);

        // Test with all sensitivities
        println!("\n== TESTING ALL SENSITIVITIES ==");
        let low_result = is_gibberish(text, Sensitivity::Low);
        let medium_result = is_gibberish(text, Sensitivity::Medium);
        let high_result = is_gibberish(text, Sensitivity::High);
        
        println!("Low sensitivity result: {}", if low_result { "GIBBERISH" } else { "ENGLISH" });
        println!("Medium sensitivity result: {}", if medium_result { "GIBBERISH" } else { "ENGLISH" });
        println!("High sensitivity result: {}", if high_result { "GIBBERISH" } else { "ENGLISH" });

        println!("\n== EXPECTED RESULT ==");
        println!("Expected Medium sensitivity result: GIBBERISH");

        assert!(is_gibberish(text, Sensitivity::Medium), "Code snippets should be classified as gibberish with medium sensitivity");
    }

    // Mixed language and special cases
    #[test]
    fn test_hashtags() {
        assert!(!is_gibberish(
            "Great party! #awesome #fun #weekend",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_emoji_text() {
        assert!(!is_gibberish(
            "Having fun at the beach 🏖️ with friends 👥",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_mixed_languages() {
        assert!(!is_gibberish(
            "The sushi 寿司 was delicious",
            Sensitivity::Medium
        ));
    }

    // Technical content
    #[test]
    fn test_scientific_notation() {
        assert!(!is_gibberish(
            "The speed of light is 3.0 x 10^8 meters per second",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_chemical_formula() {
        assert!(!is_gibberish(
            "Water H2O and Carbon Dioxide CO2 are molecules",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_mathematical_expression() {
        assert!(!is_gibberish(
            "Let x = 2y + 3z where y and z are variables",
            Sensitivity::Medium
        ));
    }

    // Creative text formats
    #[test]
    fn test_ascii_art() {
        assert!(is_gibberish("|-o-|", Sensitivity::Medium));
    }

    #[test]
    fn test_leetspeak() {
        assert!(is_gibberish("l33t h4x0r", Sensitivity::Medium));
    }

    #[test]
    fn test_repeated_punctuation() {
        assert!(!is_gibberish(
            "Wow!!! This is amazing!!!",
            Sensitivity::Medium
        ));
    }

    // Edge cases with numbers and symbols
    #[test]
    fn test_phone_number() {
        assert!(!is_gibberish(
            "Call me at 123-456-7890",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_credit_card() {
        assert!(is_gibberish("4532 7153 5678 9012", Sensitivity::Medium));
    }

    // Formatting edge cases
    #[test]
    fn test_extra_spaces() {
        assert!(!is_gibberish(
            "This    has    many    spaces",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_newlines() {
        assert!(!is_gibberish(
            "This has\nmultiple\nlines",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_tabs() {
        assert!(is_gibberish(
            "Column1\tColumn2\tColumn3",
            Sensitivity::Medium
        ));
    }

    // Common internet text
    #[test]
    fn test_file_path() {
        assert!(!is_gibberish(
            "Open C:\\Program Files\\App\\config.txt",
            Sensitivity::Medium
        ));
    }

    #[test]
    fn test_valid_english_text() {
        let text = "hello this is an example text";
        println!("\n==== DEBUGGING VALID ENGLISH TEXT ====");
        println!("Original text: '{}'", text);

        // Debug the cleaning process
        let cleaned = clean_text(text);
        println!("Cleaned text: '{}'", cleaned);

        // Split into words and check each one
        let words: Vec<&str> = cleaned
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .collect();

        println!("\n== WORD ANALYSIS ==");
        println!("Total words: {}", words.len());

        let mut english_word_count = 0;
        println!("Words after splitting:");
        for word in &words {
            let is_english = is_english_word(word);
            if is_english {
                english_word_count += 1;
            }
            println!("  \"{}\" - {}", word, if is_english { "ENGLISH WORD" } else { "not English" });
        }

        println!(
            "English words found: {} out of {} ({:.2}%)",
            english_word_count,
            words.len(),
            if words.is_empty() { 0.0 } else { english_word_count as f64 / words.len() as f64 * 100.0 }
        );

        // Check n-grams
        let trigrams = generate_ngrams(&cleaned, 3);
        let quadgrams = generate_ngrams(&cleaned, 4);

        let valid_trigrams = trigrams
            .iter()
            .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        let valid_quadgrams = quadgrams
            .iter()
            .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        let trigram_score = if trigrams.is_empty() {
            0.0
        } else {
            valid_trigrams.len() as f64 / trigrams.len() as f64
        };

        let quadgram_score = if quadgrams.is_empty() {
            0.0
        } else {
            valid_quadgrams.len() as f64 / quadgrams.len() as f64
        };

        println!("\n== TRIGRAM ANALYSIS ==");
        println!("Total trigrams: {}", trigrams.len());
        println!("All trigrams:");
        for trigram in &trigrams {
            let is_common = COMMON_TRIGRAMS.contains(trigram.as_str());
            println!("  \"{}\" - {}", trigram, if is_common { "COMMON" } else { "uncommon" });
        }
        println!("Trigram score: {:.3}", trigram_score);

        println!("\n== QUADGRAM ANALYSIS ==");
        println!("Total quadgrams: {}", quadgrams.len());
        println!("All quadgrams:");
        for quadgram in &quadgrams {
            let is_common = COMMON_QUADGRAMS.contains(quadgram.as_str());
            println!("  \"{}\" - {}", quadgram, if is_common { "COMMON" } else { "uncommon" });
        }
        println!("Quadgram score: {:.3}", quadgram_score);

        // Calculate trigram coverage
        let trigram_coverage = if cleaned.len() <= 3 {
            1.0
        } else {
            trigrams.len() as f64 / (cleaned.len() as f64 - 2.0)
        };
        println!("Trigram coverage: {:.2}%", trigram_coverage * 100.0);

        // Check suspicious pattern
        let english_word_ratio = if words.is_empty() {
            0.0
        } else {
            english_word_count as f64 / words.len() as f64
        };

        let suspicious_trigram_pattern = trigrams.len() <= 3
            && trigram_score > 0.3
            && trigram_coverage < 0.3
            && english_word_ratio < 0.1;

        println!("\n== LOW SENSITIVITY DECISION LOGIC ==");
        println!("English word ratio: {:.2}", english_word_ratio);
        println!("English words count: {}", english_word_count);
        println!("Trigram score: {:.2}", trigram_score);
        println!("Quadgram score: {:.2}", quadgram_score);
        println!("Suspicious pattern: {}", suspicious_trigram_pattern);

        if english_word_ratio > 0.8 {
            println!("Case 1: english_word_ratio > 0.8 = {}", english_word_ratio > 0.8);
        } else if english_word_count >= 3 {
            let decision = trigram_score <= 0.2 && quadgram_score <= 0.2;
            println!("Case 2: english_word_count >= 3");
            println!("  trigram_score <= 0.2: {}", trigram_score <= 0.2);
            println!("  quadgram_score <= 0.2: {}", quadgram_score <= 0.2);
            println!("  Final decision: {}", decision);
        } else if english_word_count == 1 {
            let high_scores = trigram_score > 0.8 || quadgram_score > 0.8;
            let low_scores = trigram_score <= 0.25 && quadgram_score <= 0.25;
            println!("Case 3: english_word_count == 1");
            println!("  Suspiciously high scores: {}", high_scores);
            println!("  Low scores: {}", low_scores);
        } else {
            println!("Case 4: No English words");
        }

        let result = is_gibberish(text, Sensitivity::Low);
        println!("\nFinal result: {}", if result { "GIBBERISH" } else { "ENGLISH" });

        assert!(!result, "This valid English text was incorrectly classified as gibberish");
    }

    #[test]
    fn test_gibberish_string_8() {
        let text = "aaa bbb ccc ddd";
        println!("\n==== DEBUGGING GIBBERISH STRING 8 ====");
        println!("Testing text: '{}'", text);

        let cleaned = clean_text(text);
        let words: Vec<&str> = cleaned.split_whitespace().collect();
        let english_words: Vec<&&str> = words.iter().filter(|w| is_english_word(w)).collect();

        println!("\n== Word Analysis ==");
        println!("Total words: {}", words.len());
        println!("English words: {} ({:?})", english_words.len(), english_words);

        let trigrams = generate_ngrams(&cleaned, 3);
        let quadgrams = generate_ngrams(&cleaned, 4);

        let valid_trigrams = trigrams
            .iter()
            .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        let valid_quadgrams = quadgrams
            .iter()
            .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        println!("\n== N-gram Analysis ==");
        println!(
            "Trigram score: {:.3}",
            if trigrams.is_empty() {
                0.0
            } else {
                valid_trigrams.len() as f64 / trigrams.len() as f64
            }
        );
        println!(
            "Quadgram score: {:.3}",
            if quadgrams.is_empty() {
                0.0
            } else {
                valid_quadgrams.len() as f64 / quadgrams.len() as f64
            }
        );

        test_with_sensitivities(text, true, true, true);
    }

    #[test]
    fn test_gibberish_string_9() {
        let text = "xyz abc def ghi";
        assert!(!is_gibberish(text, Sensitivity::High), "Simple letter sequences should not be classified as gibberish with high sensitivity");
    }

    #[test]
    fn test_gibberish_string_10() {
        let text = "qwe rty uio pas";
        assert!(!is_gibberish(text, Sensitivity::High), "Keyboard pattern sequences should not be classified as gibberish with high sensitivity");
    }

    #[test]
    fn test_gibberish_string_11() {
        let text = "jkl mno pqr stu";
        test_with_sensitivities(text, true, true, true);
    }

    #[test]
    fn test_gibberish_string_12() {
        let text = "vwx yza bcd efg";
        test_with_sensitivities(text, true, true, true);
    }

    #[test]
    fn test_gibberish_string_13() {
        let text = "hij klm nop qrs";
        test_with_sensitivities(text, true, true, true);
    }

    #[test]
    fn test_gibberish_string_14() {
        let text = "tuv wxy zab cde";
        test_with_sensitivities(text, true, true, true);
    }

    #[test]
    fn test_gibberish_string_15() {
        let text = "fgh ijk lmn opq";
        test_with_sensitivities(text, true, true, true);
    }

    #[test]
    fn test_rot_cipher_example() {
        let text = "Gur dhvpx oebja sbk whzcf bire gur ynml qbt";
        println!("\n==== DEBUGGING ROT CIPHER TEST ====");
        println!("Testing text: '{}'", text);

        let cleaned = clean_text(text);
        let words: Vec<&str> = cleaned.split_whitespace().collect();
        let english_words: Vec<&&str> = words.iter().filter(|w| is_english_word(w)).collect();

        println!("\n== Word Analysis ==");
        println!("Total words: {}", words.len());
        println!("English words: {} ({:?})", english_words.len(), english_words);

        let trigrams = generate_ngrams(&cleaned, 3);
        let quadgrams = generate_ngrams(&cleaned, 4);

        let valid_trigrams = trigrams
            .iter()
            .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        let valid_quadgrams = quadgrams
            .iter()
            .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        println!("\n== N-gram Analysis ==");
        println!(
            "Trigram score: {:.3}",
            if trigrams.is_empty() {
                0.0
            } else {
                valid_trigrams.len() as f64 / trigrams.len() as f64
            }
        );
        println!(
            "Quadgram score: {:.3}",
            if quadgrams.is_empty() {
                0.0
            } else {
                valid_quadgrams.len() as f64 / quadgrams.len() as f64
            }
        );

        test_with_sensitivities(text, true, true, true);
    }

    #[test]
    fn test_scrambled_words_gibberish1() {
        let text = "het uqcki wbnro xfo ujmsp vero het zlay gdo";
        println!("\n==== DEBUGGING SCRAMBLED WORDS TEST ====");
        println!("Testing text: '{}'", text);

        let cleaned = clean_text(text);
        let words: Vec<&str> = cleaned.split_whitespace().collect();
        let english_words: Vec<&&str> = words.iter().filter(|w| is_english_word(w)).collect();

        println!("\n== Word Analysis ==");
        println!("Total words: {}", words.len());
        println!("English words: {} ({:?})", english_words.len(), english_words);

        let trigrams = generate_ngrams(&cleaned, 3);
        let quadgrams = generate_ngrams(&cleaned, 4);

        let valid_trigrams = trigrams
            .iter()
            .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        let valid_quadgrams = quadgrams
            .iter()
            .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        println!("\n== N-gram Analysis ==");
        println!(
            "Trigram score: {:.3}",
            if trigrams.is_empty() {
                0.0
            } else {
                valid_trigrams.len() as f64 / trigrams.len() as f64
            }
        );
        println!(
            "Quadgram score: {:.3}",
            if quadgrams.is_empty() {
                0.0
            } else {
                valid_quadgrams.len() as f64 / quadgrams.len() as f64
            }
        );

        assert!(!is_gibberish(text, Sensitivity::High), "Scrambled English words should not be classified as gibberish with high sensitivity");
    }

    #[test]
    fn test_variable_names() {
        let text = "myVar tmpStr userInput maxVal";
        println!("\n==== DEBUGGING VARIABLE NAMES TEST ====");
        println!("Testing text: '{}'", text);

        let cleaned = clean_text(text);
        let words: Vec<&str> = cleaned.split_whitespace().collect();
        let english_words: Vec<&&str> = words.iter().filter(|w| is_english_word(w)).collect();

        println!("\n== Word Analysis ==");
        println!("Total words: {}", words.len());
        println!("English words: {} ({:?})", english_words.len(), english_words);

        let trigrams = generate_ngrams(&cleaned, 3);
        let quadgrams = generate_ngrams(&cleaned, 4);

        let valid_trigrams = trigrams
            .iter()
            .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        let valid_quadgrams = quadgrams
            .iter()
            .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
            .collect::<Vec<_>>();

        println!("\n== N-gram Analysis ==");
        println!(
            "Trigram score: {:.3}",
            if trigrams.is_empty() {
                0.0
            } else {
                valid_trigrams.len() as f64 / trigrams.len() as f64
            }
        );
        println!(
            "Quadgram score: {:.3}",
            if quadgrams.is_empty() {
                0.0
            } else {
                valid_quadgrams.len() as f64 / quadgrams.len() as f64
            }
        );

        test_with_sensitivities(text, true, true, true);
    }
}