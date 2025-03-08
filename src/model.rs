use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::fs::{self, File};
use std::io::{self, Write, Read, copy};
use thiserror::Error;
use reqwest::blocking::Client;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use log::warn;

// Candle imports
use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};

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
    
    #[error("Candle error: {0}")]
    Candle(String),
    
    #[error("Tokenizer error: {0}")]
    Tokenizer(String),
}

// Convert Candle errors to our error type
impl From<candle_core::Error> for ModelError {
    fn from(err: candle_core::Error) -> Self {
        ModelError::Candle(err.to_string())
    }
}

// Convert Tokenizer errors to our error type
impl From<tokenizers::Error> for ModelError {
    fn from(err: tokenizers::Error) -> Self {
        ModelError::Tokenizer(err.to_string())
    }
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
pub struct Model {
    model: BertModel,
    tokenizer: tokenizers::Tokenizer,
    model_path: PathBuf,
    config: ModelConfig,
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("model_path", &self.model_path)
            .field("config", &self.config)
            .finish()
    }
}

/// Static storage for loaded model
static MODEL: OnceLock<Option<Model>> = OnceLock::new();

// Model file URLs and names
const MODEL_FILES: [(&str, &str); 5] = [
    ("model.safetensors", "https://huggingface.co/gibberish-or-not/gibberish-detector/resolve/main/model.safetensors"),
    ("config.json", "https://huggingface.co/gibberish-or-not/gibberish-detector/resolve/main/config.json"),
    ("tokenizer.json", "https://huggingface.co/gibberish-or-not/gibberish-detector/resolve/main/tokenizer.json"),
    ("tokenizer_config.json", "https://huggingface.co/gibberish-or-not/gibberish-detector/resolve/main/tokenizer_config.json"),
    ("vocab.txt", "https://huggingface.co/gibberish-or-not/gibberish-detector/resolve/main/vocab.txt"),
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
                warn!("Model files not found at: {}", path.display());
                return None;
            }
            
            match Self::load(path) {
                Ok(model) => Some(model),
                Err(e) => {
                    warn!("Failed to load model: {}", e);
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

        // Load the BertConfig directly from the config.json file
        let bert_config_path = path.join("config.json");
        let bert_config: BertConfig = {
            let mut file = File::open(&bert_config_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str(&contents)
                .map_err(|e| ModelError::Json(e))?
        };

        // Load model weights using Candle
        let model_path = path.join("model.safetensors");
        let device = Device::Cpu;
        
        // Create VarBuilder from safetensors file
        let vb = if model_path.exists() {
            unsafe {
                VarBuilder::from_mmaped_safetensors(&[model_path], DType::F32, &device)
                    .map_err(|e| ModelError::Candle(e.to_string()))?
            }
        } else {
            return Err(ModelError::Model("model.safetensors not found".to_string()));
        };
        
        // Create BertModel
        let model = BertModel::load(vb, &bert_config)
            .map_err(|e| ModelError::Candle(e.to_string()))?;
            
        // Load tokenizer
        let tokenizer_path = path.join("tokenizer.json");
        let tokenizer = if tokenizer_path.exists() {
            tokenizers::Tokenizer::from_file(&tokenizer_path)
                .map_err(|e| ModelError::Tokenizer(e.to_string()))?
        } else {
            return Err(ModelError::Model("tokenizer.json not found".to_string()));
        };

        warn!("Model loaded successfully from: {}", path.display());
        Ok(Self {
            model,
            tokenizer,
            model_path: path.to_path_buf(),
            config,
        })
    }

    /// Run inference using Candle
    pub fn predict(&self, text: &str) -> bool {
        if text.is_empty() {
            return true;
        }

        match self.predict_with_score(text) {
            Ok(score) => score < 0.5, // Threshold for gibberish
            Err(e) => {
                warn!("Prediction error: {}", e);
                true // Default to gibberish on error
            }
        }
    }
    
    /// Run prediction with score
    fn predict_with_score(&self, text: &str) -> Result<f32, ModelError> {
        // Tokenize input
        let encoding = self.tokenizer.encode(text, true)
            .map_err(|e| ModelError::Tokenizer(e.to_string()))?;
        
        let input_ids = encoding.get_ids();
        let token_type_ids = encoding.get_type_ids();
        
        // Convert to tensors
        let device = Device::Cpu;
        let input_ids = Tensor::new(input_ids, &device)?;
        let token_type_ids = Tensor::new(token_type_ids, &device)?;
        
        // Run model
        let output = self.model.forward(&input_ids, &token_type_ids)?;
        
        // Apply classification head (assuming binary classification)
        // Note: This is a simplified classification head and may need adjustment
        // based on your specific model architecture
        let classifier_weights = Tensor::new(&[[1.0f32, -1.0f32]], &device)?;
        let logits = output.matmul(&classifier_weights)?;
        
        // Apply sigmoid manually since Tensor doesn't have a sigmoid method
        let prob = logits.get(0)?.get(0)?.to_scalar::<f32>()?;
        let prob = 1.0 / (1.0 + (-prob).exp());
        
        Ok(prob)
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
            warn!("File already exists, skipping: {}", filename);
            progress((i as f32 + 1.0) / MODEL_FILES.len() as f32);
            continue;
        }
        
        warn!("Downloading: {} from {}", filename, url);
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
        
        warn!("Downloaded: {}", filename);
    }
    
    // Create model info file
    let info_path = path.join("model_info.txt");
    let mut info_file = File::create(info_path)?;
    writeln!(info_file, "HuggingFace Model: gibberish-or-not/gibberish-detector")?;
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

/// Download model with a simple progress bar
pub fn download_model_with_progress_bar<P: AsRef<Path>>(path: P) -> Result<(), ModelError> {
    println!("Downloading model...");
    download_model(path, |progress| {
        let width = 50;
        let pos = (progress * width as f32) as usize;
        
        print!("\r[");
        for i in 0..width {
            if i < pos {
                print!("=");
            } else if i == pos {
                print!(">");
            } else {
                print!(" ");
            }
        }
        print!("] {:.0}%", progress * 100.0);
        let _ = io::stdout().flush();
    })?;
    println!("\nDownload complete!");
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
        
        // Write tokenizer.json (minimal version for testing)
        let tokenizer_path = test_dir.join("tokenizer.json");
        fs::write(&tokenizer_path, r#"{
            "model": {
                "type": "WordPiece",
                "vocab": {"hello": 0, "world": 1, "[UNK]": 2, "[PAD]": 3},
                "unk_token": "[UNK]"
            },
            "added_tokens": []
        }"#)?;
        
        // Write tokenizer_config.json
        let tokenizer_config_path = test_dir.join("tokenizer_config.json");
        fs::write(&tokenizer_config_path, r#"{
            "do_lower_case": true,
            "unk_token": "[UNK]",
            "pad_token": "[PAD]"
        }"#)?;

        // Create a dummy safetensors file for testing
        // In a real test, you would need to create a valid safetensors file
        let safetensors_path = test_dir.join("model.safetensors");
        fs::write(&safetensors_path, "DUMMY SAFETENSORS FILE")?;

        Ok(test_dir)
    }

    #[test]
    fn test_model_exists() -> Result<(), ModelError> {
        let model_path = setup_test_model()?;
        assert!(Model::exists(&model_path));
        Ok(())
    }

    #[test]
    fn test_default_model_path() {
        let path = default_model_path();
        assert!(path.ends_with("model"));
    }
}
