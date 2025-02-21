use phf::phf_set;
use std::collections::HashMap;

static COMMON_WORDS: phf::Set<&'static str> = phf_set! {
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "i",
    "it", "for", "not", "on", "with", "he", "as", "you", "do", "at",
    "this", "but", "his", "by", "from", "they", "we", "say", "her", "she",
    "or", "an", "will", "my", "one", "all", "would", "there", "their", "what"
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

fn calculate_letter_frequency_score(text: &str) -> f64 {
    let mut freq_map: HashMap<char, usize> = HashMap::new();
    let text_lower = text.to_lowercase();
    let total_letters: f64 = text_lower.chars()
        .filter(|c| ENGLISH_LETTERS.contains(c))
        .map(|c| {
            *freq_map.entry(c).or_insert(0) += 1;
            1
        })
        .sum::<usize>() as f64;

    if total_letters == 0.0 {
        return 0.0;
    }

    // Calculate frequency difference from English
    let freq_diff: f64 = LETTER_FREQ.iter()
        .map(|(c, expected_freq)| {
            let actual_freq = *freq_map.get(c).unwrap_or(&0) as f64 / total_letters;
            (expected_freq - actual_freq).abs()
        })
        .sum::<f64>();

    // Convert to a score (0 to 1, where 1 is perfect match)
    // Scale the difference to make the scoring more lenient
    (1.0 - freq_diff * 0.5).max(0.0)
}

fn calculate_vowel_consonant_ratio(text: &str) -> f64 {
    let mut vowels = 0;
    let mut consonants = 0;

    for c in text.to_lowercase().chars() {
        if VOWELS.contains(&c) {
            vowels += 1;
        } else if ENGLISH_LETTERS.contains(&c) {
            consonants += 1;
        }
    }

    if consonants == 0 {
        return 0.0;
    }

    // Typical English vowel/consonant ratio is around 0.4-0.6
    let ratio = vowels as f64 / consonants as f64;
    let ideal_ratio = 0.5;
    let diff = (ratio - ideal_ratio).abs();
    
    // Convert to a score (0 to 1, where 1 is perfect match)
    // More lenient scoring for vowel/consonant ratio
    if diff <= 0.3 {
        1.0 - (diff / 0.3)
    } else {
        0.0
    }
}

fn calculate_word_score(text: &str) -> f64 {
    let text_lower = text.to_lowercase();
    let words: Vec<&str> = text_lower.split_whitespace().collect();

    if words.is_empty() {
        return 0.0;
    }

    let common_word_count = words.iter()
        .filter(|word| COMMON_WORDS.contains(*word))
        .count() as f64;

    common_word_count / words.len() as f64
}

pub fn is_english(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Special handling for very short text (1-2 words)
    let word_count = trimmed.split_whitespace().count();
    if word_count <= 2 {
        // For single character, only check if it's an English letter
        if word_count == 1 && trimmed.len() == 1 {
            let ch = trimmed.chars().next().unwrap();
            return ENGLISH_LETTERS.contains(&ch) && !ch.is_numeric() && ch.is_alphabetic();
        }
        let word_score = calculate_word_score(trimmed);
        let letter_freq_score = calculate_letter_frequency_score(trimmed);
        // More lenient handling for short text
        return word_score > 0.3 || letter_freq_score > 0.5;
    }

    let bigrams = generate_ngrams(trimmed, 2);
    let trigrams = generate_ngrams(trimmed, 3);
    let quadgrams = generate_ngrams(trimmed, 4);

    if bigrams.is_empty() || trigrams.is_empty() {
        return false;
    }

    // Calculate n-gram scores
    let bigram_score = bigrams.iter()
        .filter(|gram| COMMON_BIGRAMS.contains(gram.as_str()))
        .count() as f64 / bigrams.len() as f64;

    let trigram_score = trigrams.iter()
        .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
        .count() as f64 / trigrams.len() as f64;

    let quadgram_score = if !quadgrams.is_empty() {
        quadgrams.iter()
            .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
            .count() as f64 / quadgrams.len() as f64
    } else {
        0.0
    };

    // Calculate additional scores
    let letter_freq_score = calculate_letter_frequency_score(trimmed);
    let vowel_consonant_score = calculate_vowel_consonant_ratio(trimmed);
    let word_score = calculate_word_score(trimmed);

    // Check for repetitive patterns
    let unique_words = trimmed.split_whitespace().collect::<std::collections::HashSet<_>>();
    let repetition_penalty = if (unique_words.len() as f64) / (word_count as f64) < 0.3 {
        0.5 // Significant penalty for highly repetitive text
    } else {
        1.0
    };

    // Weighted combination of all scores
    let combined_score = (
        0.15 * bigram_score +
        0.20 * trigram_score +
        0.25 * quadgram_score +
        0.15 * letter_freq_score +
        0.10 * vowel_consonant_score +
        0.15 * word_score
    ) * repetition_penalty;

    // Threshold is now more strict due to better scoring
    combined_score >= 0.20
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_text() {
        assert!(is_english("The quick brown fox jumps over the lazy dog"));
        assert!(is_english("This is a simple English sentence"));
        assert!(is_english("Programming in Rust is fun and productive"));
        assert!(is_english("The weather is beautiful today"));
    }

    #[test]
    fn test_non_english_text() {
        assert!(!is_english("xkcd vwpq mntb zzzz"));
        assert!(!is_english("12345 67890"));
        assert!(!is_english(""));
        assert!(!is_english("qwerty asdfgh zxcvbn"));
        assert!(!is_english("zzzzz xxxxx qqqqq"));
    }

    #[test]
    fn test_mixed_text() {
        assert!(is_english("HelloWorld123"));
        assert!(is_english("Hello_World"));
        assert!(!is_english("H3ll0 W0rld!!!111"));
        assert!(is_english("I have 5 apples and 3 oranges"));
        assert!(is_english("Send email to contact@example.com"));
    }

    #[test]
    fn test_edge_cases() {
        assert!(!is_english("aaaaaaaaaaaaaaa")); // Repetitive letters
        assert!(!is_english("thththththththth")); // Repetitive common bigrams
        assert!(!is_english("thethethethethe")); // Repetitive common words
        assert!(!is_english("    ")); // Only whitespace
        assert!(!is_english("!@#$%^&*()")); // Only symbols
    }

    #[test]
    fn test_short_text() {
        assert!(is_english("The cat"));
        assert!(is_english("I am"));
        assert!(!is_english("xy"));
        assert!(!is_english("a"));
    }

    #[test]
    fn test_technical_text() {
        assert!(is_english("The HTTP protocol uses TCP/IP"));
        assert!(is_english("README.md contains documentation"));
        assert!(is_english("Git repository needs to be initialized"));
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
