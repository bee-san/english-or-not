use phf::phf_set;

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
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }

    // Count English letters
    let total_chars = trimmed.chars().count() as f64;
    let english_letter_count = trimmed.chars()
        .filter(|c| ENGLISH_LETTERS.contains(c))
        .count() as f64;
    
    // If less than 50% English letters, it's not worth checking further
    if english_letter_count / total_chars < 0.5 {
        return true;
    }

    // Check for any English words
    let words: Vec<&str> = trimmed.split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphabetic()))
        .filter(|word| !word.is_empty())
        .collect();

    // If any word is English, consider it English text
    if words.iter().any(|word| is_english_word(&word.to_lowercase())) {
        return false;
    }

    // Check trigrams and quadgrams
    let trigrams = generate_ngrams(trimmed, 3);
    let quadgrams = generate_ngrams(trimmed, 4);

    let valid_trigrams = trigrams.iter()
        .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
        .count() as f64;
    
    let valid_quadgrams = quadgrams.iter()
        .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
        .count() as f64;

    // Calculate scores
    let trigram_score = if trigrams.is_empty() { 
        0.0 
    } else { 
        valid_trigrams / trigrams.len() as f64 
    };

    let quadgram_score = if quadgrams.is_empty() { 
        0.0 
    } else { 
        valid_quadgrams / quadgrams.len() as f64 
    };

    // If either score is high enough, consider it English
    if trigram_score > 0.2 || quadgram_score > 0.2 {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_text() {
        assert!(!is_gibberish("The quick brown fox jumps over the lazy dog."));
        assert!(!is_gibberish("This is a simple English sentence."));
        assert!(!is_gibberish("Hello, world!"));
        // Test single English word
        assert!(!is_gibberish("hello"));
        // Test text with common trigrams/quadgrams
        assert!(!is_gibberish("ther with tion"));
    }

    #[test]
    fn test_non_english_text() {
        assert!(is_gibberish("12345 67890"));
        assert!(is_gibberish(""));
        assert!(is_gibberish("你好世界"));
        assert!(is_gibberish("!@#$%^&*()"));
        // Text without enough English letters
        assert!(is_gibberish("123 456 789 abc"));
    }

    #[test]
    fn test_encoded_patterns() {
        assert!(is_gibberish("MOTCk4ywLLjjEE2="));
        assert!(is_gibberish("4-Fc@w7MF"));
        assert!(is_gibberish("Vszzc hvwg wg zcbu"));
        assert!(is_gibberish("buubdl"));
    }

}
