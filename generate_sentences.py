import random
import string
import requests
import os

# Constants
NUM_SENTENCES = 1000
NUM_GIBBERISH = 1000
SLANG_PROMPT = "Generate a casual English sentence using modern slang. Keep it short (under 15 words)."

def generate_english_sentences():
    sentences = []
    headers = {
        "Authorization": f"Bearer {os.getenv('openrouter')}",
        "Content-Type": "application/json"
    }
    
    for _ in range(NUM_SENTENCES):
        data = {
            "model": "huggingfaceh4/zephyr-7b-beta",
            "messages": [{"role": "user", "content": SLANG_PROMPT}]
        }
        response = requests.post(
            "https://openrouter.ai/api/v1/chat/completions",
            headers=headers,
            json=data
        )
        response.raise_for_status()
        sentences.append(response.json()['choices'][0]['message']['content'].strip())
    return sentences

def generate_gibberish():
    gibberish = []
    for _ in range(NUM_GIBBERISH):
        # Generate random string of 3-7 "words" with 3-8 characters each
        words = []
        for _ in range(random.randint(3, 7)):
            word_length = random.randint(3, 8)
            word = ''.join(random.choices(string.ascii_lowercase, k=word_length))
            words.append(word)
        gibberish.append(' '.join(words))
    return gibberish

def save_to_file(filename, data):
    with open(filename, 'w') as f:
        for item in data:
            f.write(f"{item}\n")

def main():
    print("Generating English sentences...")
    english_sentences = generate_english_sentences()
    print("Generating gibberish...")
    gibberish_sentences = generate_gibberish()
    
    save_to_file('english_sentences.txt', english_sentences)
    save_to_file('gibberish_sentences.txt', gibberish_sentences)
    print("Data generation complete!")

if __name__ == "__main__":
    main()
