import random
import string
import requests
import os
import sys
import time
import phf

# Constants
NUM_SENTENCES = 1000
NUM_GIBBERISH = 1000
SLANG_PROMPT = "Generate a casual English sentence using modern slang. Keep it short (under 15 words)."


def generate_local_english_sentences():
    """Generate simple English sentences locally as fallback"""
    subjects = ["I", "You", "We", "They", "He", "She"]
    verbs = ["like", "love", "hate", "enjoy", "prefer"]
    objects = ["apples", "music", "coding", "reading", "traveling"]

    sentences = []
    for _ in range(NUM_SENTENCES):
        sentence = f"{random.choice(subjects)} {random.choice(verbs)} {random.choice(objects)}."
        sentences.append(sentence)
    return sentences


def generate_english_sentences():
    sentences = []
    api_key = os.getenv("OPENROUTER_API_KEY")
    print(api_key)

    if not api_key:
        print(
            "Warning: OPENROUTER_API_KEY environment variable not set. Using local sentence generator."
        )
        return generate_local_english_sentences()

    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}

    for _ in range(NUM_SENTENCES):
        try:
            data = {
                "model": "openai/gpt-3.5-turbo",
                "messages": [{"role": "user", "content": SLANG_PROMPT}],
            }
            response = requests.post(
                "https://openrouter.ai/api/v1/chat/completions",
                headers=headers,
                json=data,
                timeout=30,
            )
            # Add a short delay between API calls
            time.sleep(1)
            response.raise_for_status()
            sentences.append(
                response.json()["choices"][0]["message"]["content"].strip()
            )
        except (requests.exceptions.RequestException, KeyError) as e:
            print(f"API request failed: {e}. Falling back to local sentence generator.")
            return generate_local_english_sentences()

    return sentences


def generate_gibberish():
    gibberish = []
    for _ in range(NUM_GIBBERISH):
        # Generate random string of 3-7 "words" with 3-8 characters each
        words = []
        for _ in range(random.randint(3, 7)):
            word_length = random.randint(3, 8)
            word = "".join(random.choices(string.ascii_lowercase, k=word_length))
            words.append(word)
        gibberish.append(" ".join(words))
    return gibberish


def generate_phf_map(words):
    """Generate a perfect hash function map from the given words"""
    phf_map = phf.PHF(words)
    return phf_map

def save_to_file(filename, data):
    with open(filename, "w") as f:
        for item in data:
            f.write(f"{item}\n")

def save_phf_map(filename, phf_map):
    """Save the PHF map to a Rust source file"""
    with open(filename, "w") as f:
        f.write("use phf::phf_map;\n\n")
        f.write("pub static WORD_MAP: phf::Map<&'static str, ()> = phf_map! {\n")
        for word in phf_map.keys():
            f.write(f'    "{word}" => (),\n')
        f.write("};\n")


def main():
    print("Generating English sentences...")
    english_sentences = generate_english_sentences()
    print("Generating gibberish...")
    gibberish_sentences = generate_gibberish()

    save_to_file("english_sentences.txt", english_sentences)
    save_to_file("gibberish_sentences.txt", gibberish_sentences)
    
    # Generate PHF map from English sentences
    print("Generating PHF map...")
    phf_map = generate_phf_map(english_sentences)
    save_phf_map("src/word_map.rs", phf_map)
    
    print("Data generation complete!")


if __name__ == "__main__":
    main()
