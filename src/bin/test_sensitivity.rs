use gibberish_or_not::{is_gibberish, Sensitivity};

fn main() {
    let test_strings = [
        "wjxyi yi qd unqcfbu ev iecujxydw duqj jxqj sqd ru udsetut",
        "ob353hytghof2hscotherfohnmepk6gyloe2faf2ee",
        "3hytghof2hscotherfohnmepk6gyloe2faf2ee53bo",
        "bpqa qa i mfiutm n ct ainm",
        "estd td l pilxawp q 7 dlqp",
        "bpqa qa i mfiutm n 4 ainm",
    ];

    for text in test_strings.iter() {
        let low = is_gibberish(text, Sensitivity::Low);
        let medium = is_gibberish(text, Sensitivity::Medium);
        let high = is_gibberish(text, Sensitivity::High);

        println!("Text: \"{}\"", text);
        println!("  Low: {}", low);
        println!("  Medium: {}", medium);
        println!("  High: {}", high);
        println!();
    }
}
