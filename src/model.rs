use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::fs::{self, File};
use std::io::{self, Write, Read, BufRead, copy};
use thiserror::Error;
use reqwest::blocking::Client;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Errors that can occur during model operations
#[derive(Error, Debug)]
pub enum ModelError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Model error: {0}")]
    Model(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Model configuration from config.json
#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct ModelConfig {
    vocab_size: usize,
    hidden_size: usize,
    num_attention_heads: usize,
    num_hidden_layers: usize,
    attention_head_size: usize,
    intermediate_size: usize,
    max_position_embeddings: usize,
    type_vocab_size: usize,
    layer_norm_eps: f32,
}

/// Model for enhanced gibberish detection
#[derive(Debug)]
pub struct Model {
    model_path: PathBuf,
    config: ModelConfig,
    vocab: Vec<String>,
    embeddings: Vec<f32>,
    attention_weights: Vec<f32>,
    classifier_weights: Vec<f32>,
}

/// Static storage for loaded model
static MODEL: OnceLock<Option<Model>> = OnceLock::new();

// Model file URLs and names
const MODEL_FILES: [(&str, &str); 3] = [
    ("pytorch_model.bin", "https://huggingface.co/madhurjindal/autonlp-Gibberish-Detector-492513457/resolve/main/pytorch_model.bin"),
    ("config.json", "https://huggingface.co/madhurjindal/autonlp-Gibberish-Detector-492513457/resolve/main/config.json"),
    ("vocab.txt", "https://huggingface.co/madhurjindal/autonlp-Gibberish-Detector-492513457/resolve/main/vocab.txt"),
];

impl Model {
    /// Check if model exists at given path
    pub fn exists(path: &Path) -> bool {
        if !path.exists() {
            return false;
        }
        
        // Check if all required model files exist
        for (filename, _) in MODEL_FILES.iter() {
            if !path.join(filename).exists() {
                return false;
            }
        }
        
        true
    }

    /// Get or load model singleton
    pub fn get_or_load(path: &Path) -> Option<&'static Model> {
        MODEL.get_or_init(|| {
            if !Self::exists(path) {
                log::warn!("Model files not found at: {}", path.display());
                return None;
            }
            
            match Self::load(path) {
                Ok(model) => Some(model),
                Err(e) => {
                    log::error!("Failed to load model: {}", e);
                    None
                }
            }
        }).as_ref()
    }

    /// Load model from disk
    fn load(path: &Path) -> Result<Self, ModelError> {
        // Load config
        let config_path = path.join("config.json");
        let config: ModelConfig = {
            let mut file = File::open(&config_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str(&contents)?
        };

        // Load vocabulary
        let vocab_path = path.join("vocab.txt");
        let vocab = Self::load_vocab(&vocab_path)?;

        if vocab.len() != config.vocab_size {
            return Err(ModelError::Model(format!(
                "Vocabulary size mismatch: {} != {}",
                vocab.len(),
                config.vocab_size
            )));
        }

        // Load model weights
        let weights_path = path.join("pytorch_model.bin");
        let (embeddings, attention_weights, classifier_weights) = Self::load_weights(&weights_path, &config)?;

        log::info!("Model loaded successfully from: {}", path.display());
        Ok(Self {
            model_path: path.to_path_buf(),
            config,
            vocab,
            embeddings,
            attention_weights,
            classifier_weights,
        })
    }

    /// Load vocabulary from vocab.txt
    fn load_vocab(vocab_path: &Path) -> Result<Vec<String>, ModelError> {
        let file = File::open(vocab_path)?;
        let reader = io::BufReader::new(file);
        Ok(reader.lines()
            .filter_map(|line| line.ok())
            .collect())
    }

    /// Load and parse weights from pytorch_model.bin
    fn load_weights(weights_path: &Path, config: &ModelConfig) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>), ModelError> {
        let mut file = File::open(weights_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Parse weights
        let mut weights = Vec::new();
        for chunk in buffer.chunks_exact(4) {
            if let Ok(bytes) = chunk.try_into() {
                let value = f32::from_le_bytes(bytes);
                if value.is_finite() {
                    weights.push(value);
                } else {
                    return Err(ModelError::Model("Invalid weight value found".into()));
                }
            }
        }

        // Calculate sizes for each component
        let embedding_size = config.vocab_size * config.hidden_size;
        let attention_size = config.num_hidden_layers * config.hidden_size * config.attention_head_size * 3;
        let classifier_size = config.hidden_size * 2;

        let total_size = embedding_size + attention_size + classifier_size;
        if weights.len() < total_size {
            return Err(ModelError::Model(format!(
                "Insufficient weights: got {}, need {}",
                weights.len(),
                total_size
            )));
        }

        // Split weights into components
        let embeddings = weights[..embedding_size].to_vec();
        let attention_weights = weights[embedding_size..embedding_size + attention_size].to_vec();
        let classifier_weights = weights[embedding_size + attention_size..embedding_size + attention_size + classifier_size].to_vec();

        Ok((embeddings, attention_weights, classifier_weights))
    }

    /// Run inference
    pub fn predict(&self, text: &str) -> bool {
        if text.is_empty() {
            return true;
        }

        // Tokenize input
        let mut tokens = Vec::new();
        for word in text.split_whitespace() {
            // Try exact match
            if let Some(idx) = self.vocab.iter().position(|v| v == word) {
                tokens.push(idx);
                continue;
            }

            // Try lowercase match
            let word_lower = word.to_lowercase();
            if let Some(idx) = self.vocab.iter().position(|v| v.to_lowercase() == word_lower) {
                tokens.push(idx);
                continue;
            }

            // Fallback to wordpiece tokenization
            let mut found = false;
            for (i, piece) in self.vocab.iter().enumerate() {
                if word_lower.starts_with(&piece.to_lowercase()) {
                    tokens.push(i);
                    found = true;
                    break;
                }
            }
            if !found {
                tokens.push(self.vocab.len() - 1); // [UNK] token
            }
        }

        if tokens.is_empty() {
            return true;
        }

        // Convert tokens to embeddings
        let mut hidden_states = Vec::with_capacity(tokens.len() * self.config.hidden_size);
        for &token in &tokens {
            let start = token * self.config.hidden_size;
            let end = start + self.config.hidden_size;
            if end <= self.embeddings.len() {
                hidden_states.extend_from_slice(&self.embeddings[start..end]);
            } else {
                log::error!("Token embedding index out of bounds");
                return true;
            }
        }

        // Process through attention layers
        let head_size = self.config.hidden_size / self.config.num_attention_heads;
        
        for layer in 0..self.config.num_hidden_layers {
            let mut next_states = vec![0.0; hidden_states.len()];

            // Multi-head attention
            for head in 0..self.config.num_attention_heads {
                let head_offset = layer * self.config.num_attention_heads * head_size * 3 + head * head_size;

                // Calculate attention scores
                let mut attention_scores = vec![0.0; tokens.len() * tokens.len()];
                for i in 0..tokens.len() {
                    for j in 0..tokens.len() {
                        let mut score = 0.0;
                        for k in 0..head_size {
                            let q_idx = head_offset + k;
                            let k_idx = head_offset + head_size + k;
                            if q_idx < self.attention_weights.len() && k_idx < self.attention_weights.len() {
                                score += hidden_states[i * self.config.hidden_size + k] * 
                                        self.attention_weights[q_idx] *
                                        hidden_states[j * self.config.hidden_size + k] * 
                                        self.attention_weights[k_idx];
                            }
                        }
                        attention_scores[i * tokens.len() + j] = score / (head_size as f32).sqrt();
                    }
                }

                // Softmax normalization
                for i in 0..tokens.len() {
                    let mut max_score = f32::NEG_INFINITY;
                    let mut sum = 0.0;
                    
                    for j in 0..tokens.len() {
                        let score = attention_scores[i * tokens.len() + j];
                        max_score = max_score.max(score);
                    }
                    
                    for j in 0..tokens.len() {
                        let idx = i * tokens.len() + j;
                        attention_scores[idx] = (attention_scores[idx] - max_score).exp();
                        sum += attention_scores[idx];
                    }
                    
                    for j in 0..tokens.len() {
                        let idx = i * tokens.len() + j;
                        attention_scores[idx] /= sum;
                    }
                }

                // Apply attention
                for i in 0..tokens.len() {
                    for h in 0..head_size {
                        let mut sum = 0.0;
                        for j in 0..tokens.len() {
                            let score = attention_scores[i * tokens.len() + j];
                            let v_idx = head_offset + 2 * head_size + h;
                            if v_idx < self.attention_weights.len() {
                                sum += score * hidden_states[j * self.config.hidden_size + h] * 
                                      self.attention_weights[v_idx];
                            }
                        }
                        let out_idx = i * self.config.hidden_size + head * head_size + h;
                        if out_idx < next_states.len() {
                            next_states[out_idx] = sum;
                        }
                    }
                }
            }

            // Layer normalization
            for chunk in next_states.chunks_mut(self.config.hidden_size) {
                let mut mean = 0.0;
                let mut variance = 0.0;

                // Calculate mean
                for &value in chunk.iter() {
                    mean += value;
                }
                mean /= chunk.len() as f32;

                // Calculate variance
                for &value in chunk.iter() {
                    let diff = value - mean;
                    variance += diff * diff;
                }
                variance /= chunk.len() as f32;

                // Normalize
                let std_dev = (variance + self.config.layer_norm_eps).sqrt();
                for value in chunk.iter_mut() {
                    *value = (*value - mean) / std_dev;
                }
            }

            hidden_states.copy_from_slice(&next_states);
        }

        // Global average pooling
        let mut pooled = vec![0.0; self.config.hidden_size];
        for i in 0..self.config.hidden_size {
            let mut sum = 0.0;
            let mut count = 0;
            for j in 0..tokens.len() {
                let idx = j * self.config.hidden_size + i;
                if idx < hidden_states.len() {
                    sum += hidden_states[idx];
                    count += 1;
                }
            }
            if count > 0 {
                pooled[i] = sum / count as f32;
            }
        }

        // Final classification
        let mut logits = [0.0; 2];
        for i in 0..2 {
            for j in 0..self.config.hidden_size {
                let weight_idx = i * self.config.hidden_size + j;
                if weight_idx < self.classifier_weights.len() {
                    logits[i] += pooled[j] * self.classifier_weights[weight_idx];
                }
            }
        }

        // Return true if gibberish score is higher
        logits[0] > logits[1]
    }
}

/// Download model files with progress reporting
pub fn download_model<P: AsRef<Path>>(path: P, progress: impl Fn(f32)) -> Result<(), ModelError> {
    let path = path.as_ref();
    fs::create_dir_all(path)?;
    
    let client = Client::builder()
        .timeout(Duration::from_secs(600))
        .build()?;
    
    for (i, (filename, url)) in MODEL_FILES.iter().enumerate() {
        let file_path = path.join(filename);
        
        if file_path.exists() {
            log::info!("File already exists, skipping: {}", filename);
            progress((i as f32 + 1.0) / MODEL_FILES.len() as f32);
            continue;
        }
        
        log::info!("Downloading: {} from {}", filename, url);
        progress(i as f32 / MODEL_FILES.len() as f32);
        
        let mut response = client.get(*url).send()?;
        
        if !response.status().is_success() {
            return Err(ModelError::Model(
                format!("Failed to download {}: HTTP {}", filename, response.status())
            ));
        }
        
        let content_length = response.content_length().unwrap_or(0);
        let mut file = File::create(&file_path)?;
        
        if content_length > 0 {
            let mut downloaded = 0;
            let mut buffer = [0; 8192];
            
            while let Ok(n) = response.read(&mut buffer) {
                if n == 0 { break; }
                
                file.write_all(&buffer[..n])?;
                downloaded += n;
                
                let file_progress = downloaded as f64 / content_length as f64;
                let overall_progress = (i as f32 + file_progress as f32) / MODEL_FILES.len() as f32;
                progress(overall_progress);
            }
        } else {
            copy(&mut response, &mut file)?;
            progress((i as f32 + 1.0) / MODEL_FILES.len() as f32);
        }
        
        log::info!("Downloaded: {}", filename);
    }
    
    // Create model info file
    let info_path = path.join("model_info.txt");
    let mut info_file = File::create(info_path)?;
    writeln!(info_file, "HuggingFace Model: madhurjindal/autonlp-Gibberish-Detector-492513457")?;
    writeln!(info_file, "Downloaded: {}", chrono::Local::now())?;
    writeln!(info_file, "Files:")?;
    for (filename, _) in MODEL_FILES.iter() {
        let file_path = path.join(filename);
        let file_size = file_path.metadata()?.len();
        writeln!(info_file, "  - {}: {} bytes", filename, file_size)?;
    }
    
    progress(1.0);
    Ok(())
}

/// Get default model path in user's cache directory
pub fn default_model_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("./cache"))
        .join("gibberish-or-not")
        .join("model")
}

/// Check if model exists at given path
pub fn model_exists<P: AsRef<Path>>(path: P) -> bool {
    Model::exists(path.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_model() -> Result<PathBuf, ModelError> {
        let test_dir = PathBuf::from("target").join("test_model");
        fs::create_dir_all(&test_dir)?;

        // Write config.json
        let config = ModelConfig {
            vocab_size: 4,
            hidden_size: 8,
            num_attention_heads: 2,
            num_hidden_layers: 1,
            attention_head_size: 4,
            intermediate_size: 16,
            max_position_embeddings: 512,
            type_vocab_size: 2,
            layer_norm_eps: 1e-12,
        };
        
        let config_path = test_dir.join("config.json");
        fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

        // Write vocab.txt
        let vocab_path = test_dir.join("vocab.txt");
        fs::write(&vocab_path, "hello\nworld\n[UNK]\n[PAD]")?;

        // Write test weights
        let weights_path = test_dir.join("pytorch_model.bin");
        let mut weights = Vec::new();
        
        // Calculate sizes for test weights
        let embedding_size = config.vocab_size * config.hidden_size;
        let attention_size = config.num_hidden_layers * config.hidden_size * config.attention_head_size * 3;
        let classifier_size = config.hidden_size * 2;
        
        let total_size = embedding_size + attention_size + classifier_size;
        for i in 0..total_size {
            weights.extend_from_slice(&(i as f32).to_le_bytes());
        }
        
        fs::write(&weights_path, weights)?;

        Ok(test_dir)
    }

    #[test]
    fn test_model_loading() -> Result<(), ModelError> {
        let model_path = setup_test_model()?;
        assert!(Model::exists(&model_path));
        let model = Model::load(&model_path)?;
        
        assert_eq!(model.config.vocab_size, 4);
        assert_eq!(model.vocab.len(), 4);
        assert_eq!(model.vocab[0], "hello");
        assert_eq!(model.vocab[1], "world");
        
        Ok(())
    }

    #[test]
    fn test_model_inference() -> Result<(), ModelError> {
        let model_path = setup_test_model()?;
        let model = Model::load(&model_path)?;

        // Test known words
        assert!(!model.predict("hello world"));
        
        // Test unknown words
        assert!(model.predict("xyz123 abc"));
        
        // Test empty input
        assert!(model.predict(""));
        
        // Test mixed known/unknown
        assert!(!model.predict("hello xyz"));

        Ok(())
    }

    #[test]
    fn test_model_singleton() -> Result<(), ModelError> {
        let model_path = setup_test_model()?;
        
        // First load should succeed
        let first = Model::get_or_load(&model_path).expect("Failed to load model first time");
        
        // Second load should return same instance
        let second = Model::get_or_load(&model_path).expect("Failed to load model second time");
        
        assert!(std::ptr::eq(first, second));
        Ok(())
    }
}
