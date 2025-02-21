#!/bin/bash

INPUT_FILE="words_alpha.txt"
OUTPUT_FILE="src/dictionary.rs"

echo "pub fn is_english_word(word: &str) -> bool {" > $OUTPUT_FILE
echo "    match word {" >> $OUTPUT_FILE

while read -r word; do
    # Remove any carriage return characters
    word=$(echo "$word" | tr -d '\r')
    echo "        \"$word\" => true," >> $OUTPUT_FILE
done < $INPUT_FILE

echo "        _ => false," >> $OUTPUT_FILE
echo "    }" >> $OUTPUT_FILE
echo "}" >> $OUTPUT_FILE