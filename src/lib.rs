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
        .map(|c| if ENGLISH_LETTERS.contains(&c) || c.is_ascii_digit() {
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
    // For debugging failing tests
    let is_test = text == "xgcyzw Snh fabkqta,jedm ioopl  uru v" || 
                  text == "'D<=BL C: 6@57? EI5FHN^ >I8;9 AM JCK" ||
                  text == "x,jecmdizo l  orn pg y waSuhkfubtqva";
    
    if is_test {
        println!("Analyzing text: '{}'", text);
    }
    
    // Clean the text first
    let cleaned = clean_text(text);
    
    if is_test {
        println!("Cleaned text: '{}'", cleaned);
    }
    
    // Check if empty after cleaning
    if cleaned.is_empty() {
        if is_test {
            println!("Text is empty after cleaning");
        }
        return true;
    }

    // For very short cleaned text, only check if it's an English word
    if cleaned.len() < 10 {
        let is_english = is_english_word(&cleaned);
        if is_test {
            println!("Short text (len < 10): is_english_word = {}", is_english);
        }
        return !is_english;
    }

    // Split into words and check for English words
    let words: Vec<&str> = cleaned.split_whitespace()
        .filter(|word| !word.is_empty())
        .collect();

    if is_test {
        println!("Words found: {:?}", words);
    }

    // Proceed with trigram/quadgram analysis
    let trigrams = generate_ngrams(&cleaned, 3);
    let quadgrams = generate_ngrams(&cleaned, 4);

    if is_test {
        println!("Generated trigrams: {:?}", trigrams);
        println!("Generated quadgrams: {:?}", quadgrams);
    }

    let valid_trigrams = trigrams.iter()
        .filter(|gram| COMMON_TRIGRAMS.contains(gram.as_str()))
        .collect::<Vec<_>>();
    
    let valid_quadgrams = quadgrams.iter()
        .filter(|gram| COMMON_QUADGRAMS.contains(gram.as_str()))
        .collect::<Vec<_>>();

    if is_test {
        println!("Valid trigrams: {:?}", valid_trigrams);
        println!("Valid quadgrams: {:?}", valid_quadgrams);
    }

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
    
    if is_test {
        println!("Trigram score: {}", trigram_score);
        println!("Quadgram score: {}", quadgram_score);
    }
    
    // Check for non-printable characters which are strong indicators of gibberish
    let non_printable_count = text.chars()
        .filter(|&c| c < ' ' && c != '\n' && c != '\r' && c != '\t')
        .count();
    
    if is_test && non_printable_count > 0 {
        println!("Non-printable characters found: {}", non_printable_count);
    }
    
    // If there are non-printable characters, it's likely gibberish
    if non_printable_count > 0 {
        return true;
    }

    // Check the count of English words first
    let english_words: Vec<&&str> = words.iter()
        .filter(|word| is_english_word(word))
        .collect();
    
    let english_word_count = english_words.len();
    
    if is_test {
        println!("English words found: {:?}", english_words);
        println!("English word count: {}", english_word_count);
    }
    
    // Only use ngram analysis if we have 1 English word
    if english_word_count >= 2 {
        if is_test {
            println!("Result: Not gibberish (2+ English words)");
        }
        false // Two or more English words = definitely English
    } else if english_word_count == 1 {
        // Require reasonable ngram scores
        let ngram_score_good = trigram_score > 0.15 || quadgram_score > 0.1;
        if is_test {
            println!("1 English word, ngram_score_good = {}", ngram_score_good);
            println!("Result: {}", !ngram_score_good);
        }
        !ngram_score_good
    } else {
        // No English words, just check ngram scores more strictly
        let ngram_score_good = trigram_score > 0.05 || quadgram_score > 0.03;
        if is_test {
            println!("0 English words, ngram_score_good = {}", ngram_score_good);
            println!("Result: {}", !ngram_score_good);
        }
        !ngram_score_good
    }
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

    // Edge cases
    #[test]
    fn test_single_letter() {
        assert!(is_gibberish("a"));
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

    // URLs and email addresses
    #[test]
    fn test_url() {
        assert!(!is_gibberish("Visit https://www.example.com for more info"));
    }

    #[test]
    fn test_email_address() {
        assert!(!is_gibberish("Contact us at support@example.com"));
    }

    #[test]
    fn test_url_only() {
        assert!(is_gibberish("https://aaa.bbb.ccc/ddd"));
    }

    // Code-like text
    #[test]
    fn test_variable_names() {
        assert!(is_gibberish("const myVariable = someValue"));
    }

    #[test]
    fn test_code_snippet() {
        assert!(!is_gibberish("println!({});"));
    }

    // Mixed language and special cases
    #[test]
    fn test_hashtags() {
        assert!(!is_gibberish("Great party! #awesome #fun #weekend"));
    }

    #[test]
    fn test_emoji_text() {
        assert!(!is_gibberish("Having fun at the beach 🏖️ with friends 👥"));
    }

    #[test]
    fn test_mixed_languages() {
        assert!(!is_gibberish("The sushi 寿司 was delicious"));
    }

    // Technical content
    #[test]
    fn test_scientific_notation() {
        assert!(!is_gibberish("The speed of light is 3.0 x 10^8 meters per second"));
    }

    #[test]
    fn test_chemical_formula() {
        assert!(!is_gibberish("Water H2O and Carbon Dioxide CO2 are molecules"));
    }

    #[test]
    fn test_mathematical_expression() {
        assert!(!is_gibberish("Let x = 2y + 3z where y and z are variables"));
    }

    // Creative text formats
    #[test]
    fn test_ascii_art() {
        assert!(is_gibberish("|-o-|"));
    }

    #[test]
    fn test_leetspeak() {
        assert!(is_gibberish("l33t h4x0r"));
    }

    #[test]
    fn test_repeated_punctuation() {
        assert!(!is_gibberish("Wow!!! This is amazing!!!"));
    }

    // Edge cases with numbers and symbols
    #[test]
    fn test_phone_number() {
        assert!(!is_gibberish("Call me at 123-456-7890"));
    }

    #[test]
    fn test_credit_card() {
        assert!(is_gibberish("4532 7153 5678 9012"));
    }

    // Formatting edge cases
    #[test]
    fn test_extra_spaces() {
        assert!(!is_gibberish("This    has    many    spaces"));
    }

    #[test]
    fn test_newlines() {
        assert!(!is_gibberish("This has\nmultiple\nlines"));
    }

    #[test]
    fn test_tabs() {
        assert!(is_gibberish("Column1\tColumn2\tColumn3"));
    }

    // Common internet text
    #[test]
    fn test_file_path() {
        assert!(!is_gibberish("Open C:\\Program Files\\App\\config.txt"));
    }

    #[test]
    fn test_html_tags() {
        assert!(!is_gibberish("<div class=\"container\">"));
    }

    #[test]
    fn test_json_data() {
        assert!(!is_gibberish("{\"key\": \"value\"}"));
    }

    #[test]
    fn test_base64_description() {
        assert!(!is_gibberish("Multiple base64 encodings"));
    }

    // Common passwords and usernames
    #[test]
    fn test_admin_string() {
        assert!(!is_gibberish("admin"));
    }

    #[test]
    fn test_password_qwerty() {
        assert!(!is_gibberish("qwerty"));
    }

    #[test]
    fn test_password_abc123() {
        assert!(!is_gibberish("abc123"));
    }

    #[test]
    fn test_password_password1() {
        assert!(!is_gibberish("password1"));
    }

    #[test]
    fn test_password_iloveyou() {
        assert!(!is_gibberish("iloveyou"));
    }

    #[test]
    fn test_password_numbers() {
        assert!(!is_gibberish("11111111"));
    }
    
    // Tests for strings that should be detected as gibberish
    // These are from failed decoder tests in another project
    
    #[test]
    fn test_rot47_gibberish() {
        assert!(is_gibberish("'D<=BL C: 6@57? EI5FHN^ >I8;9 AM JCK"));
    }
    
    #[test]
    fn test_binary_decoder_gibberish1() {
        assert!(is_gibberish("\u{3} \u{e}@:\u{1}`\u{7}\u{18}\u{e}@/\u{1}<\u{e}p;An\u{2}p\u{19}`o\u{3}<\u{c}p6\u{1}J\u{2}p\u{18}`o\u{3}\r"));
    }
    
    #[test]
    fn test_railfence_gibberish() {
        assert!(is_gibberish("xgcyzw Snh fabkqta,jedm ioopl  uru v"));
    }
    
    #[test]
    fn test_binary_decoder_gibberish2() {
        assert!(is_gibberish("\0*\0\u{1a}\0\r\u{10}\u{7}\u{18}\u{1}\0\u{1}R\0s\0\u{10}\0\u{18}`\rp\u{6}p\u{3}X\u{1}^\0l\0:@\u{1d}\0\u{c}P\u{6} \u{1}\u{e}"));
    }
    
    #[test]
    fn test_astar_gibberish() {
        assert!(is_gibberish(")W?:!|.b"));
    }
    
    #[test]
    fn test_railfence_gibberish2() {
        assert!(is_gibberish("x,jecmdizo l  orn pg y waSuhkfubtqva"));
    }
}
