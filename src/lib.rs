use phf::phf_set;
use std::collections::HashMap;

mod dictionary;

fn is_english_word(word: &str) -> bool {
    dictionary::ENGLISH_WORDS.contains(word)
}

// The dictionary module provides a perfect hash table implementation
// using the phf crate, which is generated at compile time
// for optimal performance and memory efficiency



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

static COMMON_BIGRAMS: phf::Set<&'static str> = phf_set! {
    "th", "he", "in", "er", "an", "re", "on", "at", "en", "nd",
    "ti", "es", "or", "te", "of", "ed", "is", "it", "al", "ar",
    "st", "to", "nt", "ng", "se", "ha", "as", "ou", "io", "le",
    "ve", "co", "me", "de", "hi", "ri", "ro", "ic", "ne", "ea",
    "ra", "ce", "li", "ch", "ll", "be", "ma", "si", "om", "ur"
};

static ENGLISH_LETTERS: phf::Set<char> = phf_set! {
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'
};

static VOWELS: phf::Set<char> = phf_set! {
    'a', 'e', 'i', 'o', 'u', 'A', 'E', 'I', 'O', 'U'
};

// English letter frequency (from most common to least common)
static LETTER_FREQ: [(char, f64); 26] = [
    ('e', 0.1202), ('t', 0.0910), ('a', 0.0812), ('o', 0.0768), ('i', 0.0731),
    ('n', 0.0695), ('s', 0.0628), ('r', 0.0602), ('h', 0.0592), ('d', 0.0432),
    ('l', 0.0398), ('u', 0.0288), ('c', 0.0271), ('m', 0.0261), ('f', 0.0230),
    ('y', 0.0211), ('w', 0.0209), ('g', 0.0203), ('p', 0.0182), ('b', 0.0149),
    ('v', 0.0111), ('k', 0.0069), ('x', 0.0017), ('q', 0.0011), ('j', 0.0010),
    ('z', 0.0007)
];

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


fn calculate_word_score(text: &str) -> f64 {
    let text_lower = text.to_lowercase();
    let words: Vec<&str> = text_lower.split_whitespace().collect();

    if words.is_empty() {
        return 0.0;
    }

    let valid_word_count = words.iter()
        .filter(|word| is_english_word(word))
        .count() as f64;

    valid_word_count / words.len() as f64
}

pub fn is_gibberish(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;  // Empty text is now considered gibberish
    }

    // If text contains any non-English characters (except spaces and basic punctuation), it's gibberish
    if text.chars().any(|c| {
        !ENGLISH_LETTERS.contains(&c) && 
        !c.is_whitespace() && 
        !matches!(c, '.' | ',' | '!' | '?' | '\'' | '"' | ';' | ':' | '-')
    }) {
        return true;
    }

    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.is_empty() {
        return true;
    }

    // Require a high percentage of valid English words
    let valid_word_count = words.iter()
        .filter(|word| {
            // Remove punctuation for word lookup
            let clean_word = word.trim_matches(|c| !c.is_alphabetic());
            !clean_word.is_empty() && is_english_word(clean_word.to_lowercase().as_str())
        })
        .count();

    // Require at least 80% of words to be valid English
    if (valid_word_count as f64 / words.len() as f64) < 0.8 {
        return true;
    }

    // Check for repetitive patterns that might indicate encoding
    let char_vec: Vec<char> = trimmed.chars()
        .filter(|c| c.is_alphabetic())
        .collect();
    
    if char_vec.len() >= 3 {
        // Check for repeated characters
        let repeated_chars = char_vec.windows(2)
            .filter(|pair| pair[0] == pair[1])
            .count() as f64 / (char_vec.len() as f64);
        
        if repeated_chars > 0.2 {
            return true;
        }

        // Check for shifted patterns (like ROT13)
        let shifted_pattern_score = char_vec.windows(2)
            .filter(|pair| {
                let diff = (pair[0] as i32 - pair[1] as i32).abs();
                diff == 13 || diff == 1 || diff == 5
            })
            .count() as f64 / (char_vec.len() as f64);

        if shifted_pattern_score > 0.25 {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_text() {
        assert!(is_gibberish("The quick brown fox jumps over the lazy dog"));
        assert!(is_gibberish("This is a simple English sentence"));
        assert!(is_gibberish("Programming in Rust is fun and productive"));
        assert!(is_gibberish("The weather is beautiful today"));
    }

    #[test]
    fn test_non_english_text() {
        assert!(!is_gibberish("xkcd vwpq mntb zzzz"));
        assert!(!is_gibberish("12345 67890"));
        assert!(!is_gibberish(""));
        assert!(!is_gibberish("qwerty asdfgh zxcvbn"));
        assert!(!is_gibberish("zzzzz xxxxx qqqqq"));
    }

    #[test]
    fn test_mixed_text() {
        assert!(is_gibberish("Hello World"));
        assert!(!is_gibberish("Hello_World")); // Underscores are no longer valid
        assert!(!is_gibberish("H3ll0 W0rld!!!111"));
        assert!(is_gibberish("I have apples and oranges"));
        assert!(is_gibberish("Send email to contact@example.com"));
    }

    #[test]
    fn test_edge_cases() {
        assert!(!is_gibberish("aaaaaaaaaaaaaaa")); // Repetitive letters
        assert!(!is_gibberish("thththththththth")); // Repetitive common bigrams
        assert!(!is_gibberish("thethethethethe")); // Repetitive common words
        assert!(!is_gibberish("    ")); // Only whitespace
        assert!(!is_gibberish("!@#$%^&*()")); // Only symbols
    }

    #[test]
    fn test_encoded_patterns() {
        assert!(is_gibberish("MOTCk4ywLLjjEE2=")); // Base64-like gibberish
        assert!(is_gibberish("4\u{19}-Fc<@w7\\MF\u{4}")); // Random gibberish with control chars
        assert!(is_gibberish("Vszzc! hvwg wg zcbu hslh?")); // ROT-style encoding
        assert!(is_gibberish("buubdl")); // Encoded text
        assert!(is_gibberish("gzzgiq")); 
        assert!(is_gibberish("cvvcem")); 
        assert!(is_gibberish("vszzc hvwg wg zcbu hslh")); 
        assert!(is_gibberish("agoykxtwpS,ceh fmzibuqo lauj nrdv   ")); 
        assert!(is_gibberish("=EjLw4CO2EjLykTM")); 
    }

    #[test]
    fn test_short_text() {
        assert!(is_gibberish("The cat"));
        assert!(is_gibberish("I am"));
        assert!(!is_gibberish("xy"));
        assert!(!is_gibberish("a")); // Single letters are not considered valid words
        assert!(is_gibberish("Hello"));
        assert!(is_gibberish("it")); // Common two-letter word
    }

    #[test]
    fn test_technical_text() {
        assert!(is_gibberish("The HTTP protocol uses TCP/IP"));
        assert!(is_gibberish("README.md contains documentation"));
        assert!(is_gibberish("Git repository needs to be initialized"));
    }

    #[test]
    fn test_control_characters() {
        let control_chars = "\0\0\0\0\0\u{1}\u{1}\0\u{1}\0\0\0\0\0\0\0\0\u{1}\u{1}\u{1}\0\u{1}";
        assert!(is_gibberish(control_chars));

        let long_control_sequence = "\u{1}\0\u{1}\0\0\u{1}\u{1}\u{1}\u{1}\u{1}\0\0\0\0\u{1}\u{1}\0\u{1}\0\0\0\u{1}\u{1}\0\u{1}\0\0\u{1}\u{1}\u{1}\0\u{1}\u{1}\u{1}\0\u{1}\u{1}\u{1}\u{1}\0\0\0\0\u{1}\0\0\0\0\0\u{1}\u{1}\0\u{1}\u{1}\u{1}\u{1}\u{1}\u{1}\0\0\u{1}\u{1}\0\0\u{1}\0\0\0\0\0\u{1}\u{1}\0\0\0\u{1}\0\u{1}\u{1}\0\u{1}\u{1}\0\0\u{1}\u{1}\0\0\0\0\u{1}\u{1}\u{1}\0\0\0\u{1}\u{1}\u{1}\u{1}\0\u{1}\0\u{1}\u{1}\0\u{1}\0\0\0\0\0\u{1}\u{1}\u{1}\0\0\0\u{1}\u{1}\u{1}\u{1}\0\u{1}\0\u{1}\u{1}\u{1}\0\0\0\0\u{1}\u{1}\u{1}\u{1}\0\0\u{1}\0\u{1}\u{1}\u{1}\0\u{1}\0\0\u{1}\u{1}\u{1}\u{1}\0\u{1}\0\0\u{1}\0\u{1}\u{1}\0\0\0\u{1}\0\0\0\0\0\u{1}\u{1}\0\u{1}\0\u{1}\0\u{1}\u{1}\u{1}\0\u{1}\0\u{1}\u{1}\u{1}\0\0\u{1}\0\0\u{1}\u{1}\0\0\u{1}\u{1}\u{1}\u{1}\u{1}\0\0\u{1}\0\u{1}\0\u{1}\0\0\0\0\0\u{1}\u{1}\0\u{1}\u{1}\0\u{1}\u{1}\u{1}\u{1}\u{1}\0\0\u{1}\0\u{1}\0\0\0\0\0\u{1}\u{1}\u{1}\0\u{1}\u{1}\0\u{1}\u{1}\0\u{1}\u{1}\u{1}\u{1}\u{1}\u{1}\u{1}\0\u{1}\u{1}\u{1}\0\u{1}\0\u{1}\u{1}\u{1}\0";
        assert!(is_gibberish(long_control_sequence)); // This sequence should be marked as gibberish
    }

    #[test]
    fn test_complex_control_sequences() {
        let sequence1 = "\u{3} \u{e}@:\u{1}`\u{7}\u{18}\u{e}@/\u{1}<\u{e}p;An\u{2}p\u{19}`o\u{3}<\u{c}p6\u{1}J\u{2}p\u{18}`o\u{3}\r";
        assert!(is_gibberish(sequence1));

        let sequence2 = "\0*\0\u{1a}\0\r\u{10}\u{7}\u{18}\u{1}\0\u{1}R\0s\0\u{10}\0\u{18}`\rp\u{6}p\u{3}X\u{1}^\0l\0:@\u{1d}\0\u{c}P\u{6} \u{1}\u{e}";
        assert!(is_gibberish(sequence2));
    }

    #[test]
    fn test_non_english_characters() {
        assert!(!is_gibberish("你好世界")); // Chinese
        assert!(!is_gibberish("こんにちは")); // Japanese
        assert!(!is_gibberish("★☆♠♣♥♦")); // Symbols
        assert!(!is_gibberish("12345")); // Only numbers
    }

    #[test]
    fn test_letter_frequency() {
        let score = calculate_letter_frequency_score("The quick brown fox jumps over the lazy dog");
        assert!(score > 0.5); // Should have good letter frequency match

        let bad_score = calculate_letter_frequency_score("zzzzzxxxxx");
        assert!(bad_score < 0.3); // Should have poor letter frequency match
    }

    #[test]
    fn test_vowel_consonant_ratio() {
        let score = calculate_vowel_consonant_ratio("The quick brown fox");
        assert!(score > 0.7); // Should have good vowel/consonant ratio

        let bad_score = calculate_vowel_consonant_ratio("rhythm myth gym");
        assert!(bad_score < 0.5); // Too few vowels
    }
}
