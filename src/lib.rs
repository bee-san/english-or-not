use phf::phf_set;

mod dictionary;

fn is_english_word(word: &str) -> bool {
    dictionary::ENGLISH_WORDS.contains(word)
}

// The dictionary module provides a perfect hash table implementation
// using the phf crate, which is generated at compile time
// for optimal performance and memory efficiency

/// Checks if the given text is gibberish based on English word presence
/// and n-gram analysis scores. 
/// 
/// # Algorithm Steps
/// 
/// 1. Clean and normalize the input text
/// 2. Short text (len < 10) - single word check
/// 3. Split into words and count English words:
///    - 2+ English words → considered valid
///    - 1 English word → check n-gram scores
///    - 0 English words → more lenient n-gram check
/// 4. Use different n-gram thresholds depending on case

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
        .map(|c| if ENGLISH_LETTERS.contains(&c) {
            c.to_ascii_lowercase()
        } else if c.is_whitespace() {
            ' '
        } else {
            ' '
        })
        .collect()
}

fn generate_ngrams(text: &str, n: usize) -> Vec<String> {
    let filtered: String = text.to_lowercase()
        .chars()
        .map(|ch| if ENGLISH_LETTERS.contains(&ch) || ch.is_numeric() { ch } else { ' ' })
        .collect();

    filtered.split_whitespace()
        .flat_map(|word| {
            word.as_bytes()
                .windows(n)
                .filter_map(|window| String::from_utf8(window.to_vec()).ok())
        })
        .collect()
}



pub fn is_gibberish(text: &str) -> bool {
    println!("Analyzing text: '{}'", text);
    
    // Clean the text first
    let cleaned = clean_text(text);
    println!("Cleaned text: '{}'", cleaned);
    
    // Check if empty after cleaning
    if cleaned.is_empty() {
        println!("Text is empty after cleaning");
        return true;
    }

    // For very short cleaned text, only check if it's an English word
    if cleaned.len() < 10 {
        let is_english = is_english_word(&cleaned);
        println!("Short text: checking if '{}' is English word: {}", cleaned, is_english);
        return !is_english;
    }

    // Split into words and check for English words
    let words: Vec<&str> = cleaned.split_whitespace()
        .filter(|word| !word.is_empty())
        .collect();
    println!("Words found: {:?}", words);

    let has_english_word = words.iter().any(|word| {
        let is_english = is_english_word(word);
        if is_english {
            println!("Found English word: '{}'", word);
        }
        is_english
    });

    // Proceed with trigram/quadgram analysis
    let trigrams = generate_ngrams(&cleaned, 3);
    let quadgrams = generate_ngrams(&cleaned, 4);

    let valid_trigrams = trigrams.iter()
        .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
        .count() as f64;
    
    let valid_quadgrams = quadgrams.iter()
        .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
        .count() as f64;

    // Calculate scores
    let trigram_score = if trigrams.is_empty() { 
        println!("No trigrams found");
        0.0 
    } else { 
        let score = valid_trigrams / trigrams.len() as f64;
        println!("Trigram analysis: {} valid out of {} total (score: {:.2})", 
                valid_trigrams, trigrams.len(), score);
        score
    };

    let quadgram_score = if quadgrams.is_empty() { 
        println!("No quadgrams found");
        0.0 
    } else { 
        let score = valid_quadgrams / quadgrams.len() as f64;
        println!("Quadgram analysis: {} valid out of {} total (score: {:.2})", 
                valid_quadgrams, quadgrams.len(), score);
        score
    };

    // Check the count of English words first
    let english_word_count = words.iter()
        .filter(|word| is_english_word(word))
        .count();
    
    // Only use ngram analysis if we have 1 English word
    let result = if english_word_count >= 2 {
        false // Two or more English words = definitely English
    } else if english_word_count == 1 {
        // Require reasonable ngram scores
        let ngram_score_good = trigram_score > 0.15 || quadgram_score > 0.1;
        !ngram_score_good
    } else {
        // No English words, just check ngram scores more leniently
        let ngram_score_good = trigram_score > 0.1 || quadgram_score > 0.05;
        !ngram_score_good
    };
    println!("Final decision: text is {} (has_english_word={})", 
             if result { "gibberish" } else { "English" },
             has_english_word);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valid English text tests
    #[test]
    fn test_pangram() {
        assert!(!is_gibberish("The quick brown fox jumps over the lazy dog."));
    }

    #[test]
    fn test_simple_sentence() {
        assert!(!is_gibberish("This is a simple English sentence."));
    }

    #[test]
    fn test_hello_world() {
        assert!(!is_gibberish("Hello, world!"));
    }

    #[test]
    fn test_single_word() {
        assert!(!is_gibberish("hello"));
    }

    #[test]
    fn test_common_ngrams() {
        assert!(!is_gibberish("ther with tion"));
    }

    #[test]
    fn test_technical_text() {
        assert!(!is_gibberish("The function returns a boolean value."));
    }

    #[test]
    fn test_mixed_case() {
        assert!(!is_gibberish("MiXeD cAsE text IS still English"));
    }

    #[test]
    fn test_with_punctuation() {
        assert!(!is_gibberish("Hello! How are you? I'm doing well."));
    }

    #[test]
    fn test_long_text() {
        assert!(!is_gibberish("This is a longer piece of text that contains multiple sentences and should definitely be recognized as valid English content."));
    }

    // Gibberish text tests
    #[test]
    fn test_numbers_only() {
        assert!(is_gibberish("12345 67890"));
    }

    #[test]
    fn test_empty_string() {
        assert!(is_gibberish(""));
    }

    #[test]
    fn test_non_english_chars() {
        assert!(is_gibberish("你好世界"));
    }

    #[test]
    fn test_special_chars() {
        assert!(is_gibberish("!@#$%^&*()"));
    }

    #[test]
    fn test_base64_like() {
        assert!(is_gibberish("MOTCk4ywLLjjEE2="));
    }

    #[test]
    fn test_short_gibberish() {
        assert!(is_gibberish("4-Fc@w7MF"));
    }

    #[test]
    fn test_letter_substitution() {
        assert!(is_gibberish("Vszzc hvwg wg zcbu"));
    }

    #[test]
    fn test_repeated_chars() {
        assert!(is_gibberish("aaaaaa bbbbbb"));
    }

    // Edge cases
    #[test]
    fn test_single_letter() {
        assert!(!is_gibberish("a"));
    }

    #[test]
    fn test_mixed_valid_invalid() {
        assert!(!is_gibberish("hello xkcd world"));
    }

    #[test]
    fn test_common_abbreviation() {
        assert!(!is_gibberish("NASA FBI CIA"));
    }

    #[test]
    fn test_with_numbers() {
        assert!(!is_gibberish("Room 101 is down the hall"));
    }

    #[test]
    fn test_keyboard_mash() {
        assert!(is_gibberish("asdfgh jkl"));
    }

    #[test]
    fn test_repeated_word() {
        assert!(!is_gibberish("buffalo buffalo buffalo"));
    }
}
