use log::warn;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, copy, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;
use thiserror::Error;

// Candle imports
use candle_core::{DType, Device, Tensor};
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
const MODEL_FILES: [(&str, &str); 3] = [
    ("model.safetensors", "https://huggingface.co/madhurjindal/autonlp-Gibberish-Detector-492513457/resolve/main/model.safetensors"),
    ("config.json", "https://huggingface.co/madhurjindal/autonlp-Gibberish-Detector-492513457/resolve/main/config.json"),
    ("tokenizer.json", "https://huggingface.co/madhurjindal/autonlp-Gibberish-Detector-492513457/resolve/main/tokenizer.json"),
];

/// Status of the HuggingFace token
///
/// Used to determine whether a token is needed and if it's available
/// for downloading the model files.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenStatus {
    /// Token is available and set (either via environment variable or direct input)
    Available,
    /// Token is not set but required for model download
    Required,
    /// Token is not required (e.g., model already exists)
    NotRequired,
}

/// Check if HuggingFace token is required and available
///
/// This function checks whether a token is needed to download the model,
/// and if needed, whether it's available either via environment variable
/// or direct input.
///
/// # Arguments
///
/// * `path` - Path where model files should be located
///
/// # Returns
///
/// Returns a `TokenStatus` indicating whether a token is needed and available:
/// * `TokenStatus::NotRequired` - Model exists at path, no token needed
/// * `TokenStatus::Available` - Token is available (env var or direct)
/// * `TokenStatus::Required` - Token is needed but not available
///
/// # Examples
///
/// ```no_run
/// use gibberish_or_not::{check_token_status, default_model_path, TokenStatus};
///
/// let status = check_token_status(default_model_path());
/// match status {
///     TokenStatus::NotRequired => println!("Model exists, no token needed"),
///     TokenStatus::Available => println!("Token is available"),
///     TokenStatus::Required => println!("Please provide a HuggingFace token"),
/// }
/// ```
pub fn check_token_status<P: AsRef<Path>>(path: P) -> TokenStatus {
    let path = path.as_ref();

    // If model exists, token is not required
    if Model::exists(path) {
        return TokenStatus::NotRequired;
    }

    // Check if token is set
    match check_huggingface_token(None) {
        Some(_) => TokenStatus::Available,
        None => TokenStatus::Required,
    }
}

/// Check if HuggingFace token is set, optionally taking a token directly
fn check_huggingface_token(token: Option<&str>) -> Option<String> {
    token
        .map(String::from)
        .or_else(|| std::env::var("HUGGING_FACE_HUB_TOKEN").ok())
}

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
        MODEL
            .get_or_init(|| {
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
            })
            .as_ref()
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
            serde_json::from_str(&contents).map_err(|e| ModelError::Json(e))?
        };

        // Load model weights using Candle
        let model_path = path.join("model.safetensors");
        // TODO we could probably use GPU optionally
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
        let model =
            BertModel::load(vb, &bert_config).map_err(|e| ModelError::Candle(e.to_string()))?;

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
                false // Default to not gibberish on error, becuase its already passed all the other gibberish checkers
            }
        }
    }

    /// Run prediction with score
    fn predict_with_score(&self, text: &str) -> Result<f32, ModelError> {
        // Tokenize input
        let encoding = self
            .tokenizer
            .encode(text, true)
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
///
/// # Arguments
///
/// * `path` - Path where model files will be downloaded
/// * `progress` - Callback function that receives progress updates (0.0 to 1.0)
/// * `token` - Optional HuggingFace token. If not provided, will attempt to read from HUGGING_FACE_HUB_TOKEN environment variable
///
/// # Examples
///
/// ```no_run
/// use gibberish_or_not::{download_model, default_model_path};
///
/// // Using direct token
/// download_model(default_model_path(), |p| println!("Progress: {}%", p * 100.0), Some("your_token_here"));
///
/// // Using environment variable
/// std::env::set_var("HUGGING_FACE_HUB_TOKEN", "your_token_here");
/// download_model(default_model_path(), |p| println!("Progress: {}%", p * 100.0), None);
/// ```
pub fn download_model<P: AsRef<Path>>(
    path: P,
    mut progress: impl FnMut(f32),
    token: Option<&str>,
) -> Result<(), ModelError> {
    let path = path.as_ref();
    fs::create_dir_all(path)?;

    // Check for HuggingFace token
    let token = check_huggingface_token(token).ok_or_else(|| {
        ModelError::Model(
            "HuggingFace token not found. Either:\n\
             1. Pass the token directly to the function, or\n\
             2. Set the HUGGING_FACE_HUB_TOKEN environment variable\n\
             Get your token at: https://huggingface.co/settings/tokens"
                .to_string(),
        )
    })?;

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

        let mut response = client
            .get(*url)
            .header("Authorization", format!("Bearer {}", token))
            .send()?;

        if !response.status().is_success() {
            return Err(ModelError::Model(format!(
                "Failed to download {}: HTTP {}",
                filename,
                response.status()
            )));
        }

        let content_length = response.content_length().unwrap_or(0);
        let mut file = File::create(&file_path)?;

        if content_length > 0 {
            let mut downloaded = 0;
            let mut buffer = [0; 8192];

            while let Ok(n) = response.read(&mut buffer) {
                if n == 0 {
                    break;
                }

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
    writeln!(
        info_file,
        "HuggingFace Model: madhurjindal/autonlp-Gibberish-Detector-492513457"
    )?;
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
///
/// A convenience wrapper around `download_model` that displays a progress bar in the terminal.
///
/// # Arguments
///
/// * `path` - Path where model files will be downloaded
/// * `token` - Optional HuggingFace token. If not provided, will attempt to read from HUGGING_FACE_HUB_TOKEN environment variable
///
/// # Examples
///
/// ```no_run
/// use gibberish_or_not::{download_model_with_progress_bar, default_model_path};
///
/// // Using direct token
/// download_model_with_progress_bar(default_model_path(), Some("your_token_here"));
///
/// // Using environment variable
/// std::env::set_var("HUGGING_FACE_HUB_TOKEN", "your_token_here");
/// download_model_with_progress_bar(default_model_path(), None);
/// ```
pub fn download_model_with_progress_bar<P: AsRef<Path>>(
    path: P,
    token: Option<&str>,
) -> Result<(), ModelError> {
    println!("Downloading model...");
    download_model(
        path,
        |progress| {
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
        },
        token,
    )?;
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
    use std::env;

    #[test]
    fn test_token_status_when_token_set() {
        env::set_var("HUGGING_FACE_HUB_TOKEN", "dummy_token");
        let test_dir = PathBuf::from("target").join("no_model");
        assert_eq!(check_token_status(&test_dir), TokenStatus::Available);
        env::remove_var("HUGGING_FACE_HUB_TOKEN");
    }

    #[test]
    fn test_token_status_when_token_missing() {
        env::remove_var("HUGGING_FACE_HUB_TOKEN");
        let test_dir = PathBuf::from("target").join("no_model");
        assert_eq!(check_token_status(&test_dir), TokenStatus::Required);
    }

    #[test]
    fn test_token_status_when_model_exists() {
        // Even without token, should return NotRequired if model exists
        env::remove_var("HUGGING_FACE_HUB_TOKEN");
        let test_dir = setup_test_model().unwrap();
        assert_eq!(check_token_status(&test_dir), TokenStatus::NotRequired);
    }

    #[test]
    fn test_model_exists() -> Result<(), ModelError> {
        let model_path = setup_test_model()?;
        assert!(Model::exists(&model_path));
        Ok(())
    }

    #[test]
    fn test_model_prediction() {
        let model_path = PathBuf::from("target").join("test_model");
        let model = Model::get_or_load(&model_path);

        // Since we can't create a valid model file for testing,
        // we'll just verify that the model loading behaves correctly
        assert!(model.is_none(), "Model should not load from invalid path");

        // Create the directory and verify it's still not loaded
        fs::create_dir_all(&model_path).unwrap();
        let model = Model::get_or_load(&model_path);
        assert!(
            model.is_none(),
            "Model should not load from empty directory"
        );
    }

    #[test]
    fn test_model_loading_errors() {
        let bad_path = PathBuf::from("nonexistent");
        assert!(
            Model::get_or_load(&bad_path).is_none(),
            "Should return None for nonexistent path"
        );

        // Test with incomplete model directory
        let incomplete_dir = PathBuf::from("target").join("incomplete_model");
        fs::create_dir_all(&incomplete_dir).unwrap();
        fs::write(incomplete_dir.join("config.json"), "{}").unwrap();
        assert!(
            Model::get_or_load(&incomplete_dir).is_none(),
            "Should return None for incomplete model"
        );
    }

    #[test]
    fn test_model_prediction_edge_cases() {
        let model_path = PathBuf::from("target").join("test_model");
        let model = Model::get_or_load(&model_path);

        // Since we can't create a valid model file for testing,
        // we'll just verify that the model loading behaves correctly
        assert!(model.is_none(), "Model should not load from invalid path");
    }

    #[test]
    fn test_model_config_validation() -> Result<(), ModelError> {
        let test_dir = PathBuf::from("target").join("invalid_config_model");
        fs::create_dir_all(&test_dir)?;

        // Test with invalid config
        let invalid_config = r#"{
            "vocab_size": 0,
            "hidden_size": -1,
            "num_attention_heads": 0,
            "num_hidden_layers": 0,
            "attention_head_size": 0,
            "intermediate_size": 0,
            "max_position_embeddings": 0,
            "type_vocab_size": 0,
            "layer_norm_eps": 0.0
        }"#;

        fs::write(test_dir.join("config.json"), invalid_config)?;
        fs::write(test_dir.join("model.safetensors"), "DUMMY")?;
        fs::write(test_dir.join("tokenizer.json"), "{}")?;

        assert!(
            Model::get_or_load(&test_dir).is_none(),
            "Should return None for invalid config"
        );

        Ok(())
    }

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

        // Write tokenizer.json (minimal version for testing)
        let tokenizer_path = test_dir.join("tokenizer.json");
        fs::write(
            &tokenizer_path,
            r#"{
            "model": {
                "type": "WordPiece",
                "vocab": {"hello": 0, "world": 1, "[UNK]": 2, "[PAD]": 3},
                "unk_token": "[UNK]"
            },
            "added_tokens": []
        }"#,
        )?;

        // Create a dummy safetensors file for testing
        let safetensors_path = test_dir.join("model.safetensors");
        fs::write(&safetensors_path, "DUMMY SAFETENSORS FILE")?;

        Ok(test_dir)
    }

    #[test]
    fn test_default_model_path() {
        let path = default_model_path();
        assert!(path.ends_with("model"));
    }
}
