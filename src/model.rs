use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::fs::{self, File};
use std::io::{self, Write, Read, copy, stdout};
use thiserror::Error;
use reqwest::blocking::Client;
use std::time::Duration;

/// Errors that can occur during model operations
///
/// This enum represents the various error types that can occur when working with the model,
/// including IO errors, network errors, model-specific errors, and JSON parsing errors.
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

/// Model for enhanced gibberish detection
pub struct Model {
    // Model files and configuration
    model_path: PathBuf,
    // We'll use a simple approach for now - in the future this could use tch-rs
}

/// Static storage for loaded model
static MODEL: OnceLock<Option<Model>> = OnceLock::new();

// Model file URLs
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
            
            // Load model only when first needed
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
        // In a real implementation, we would load the model using tch-rs here
        // For now, we'll just verify the files exist
        for (filename, _) in MODEL_FILES.iter() {
            let file_path = path.join(filename);
            if !file_path.exists() {
                return Err(ModelError::Model(format!("Required model file not found: {}", filename)));
            }
        }

        log::info!("Model loaded from: {}", path.display());
        Ok(Self { model_path: path.to_path_buf() })
    }

    /// Run inference
    pub fn predict(&self, text: &str) -> bool {
        // In a real implementation, we would use tch-rs to run inference
        // For now, we'll use a simple heuristic based on the text

        // Log that we're using the model
        log::debug!("Using model from: {}", self.model_path.display());
        
        // This is a placeholder - in reality, we would load the model and run inference
        // The actual implementation would use the transformer model
        
        // For demonstration purposes, we'll use a simple heuristic:
        // - If the text has many non-alphabetic characters, it's likely gibberish
        // - If the text has many repeated characters, it's likely gibberish
        
        let alpha_ratio = text.chars().filter(|c| c.is_alphabetic()).count() as f64 / text.len() as f64;
        let unique_chars = text.chars().collect::<std::collections::HashSet<_>>().len();
        let unique_ratio = unique_chars as f64 / text.len() as f64;
        
        // These thresholds are arbitrary and would be replaced by actual model inference
        alpha_ratio < 0.5 || unique_ratio < 0.3
    }
}

/// Download model files with progress reporting
///
/// This function downloads the required model files to the specified path and reports progress
/// through a callback function. It's designed to be used by applications that want to provide
/// custom progress reporting UI.
///
/// # Arguments
///
/// * `path` - The path where the model files should be downloaded
/// * `progress` - A callback function that receives progress updates as a float between 0.0 and 1.0
///
/// # Returns
///
/// * `Ok(())` if the download was successful
/// * `Err(ModelError)` if an error occurred during download
///
/// # Example
///
/// ```rust,no_run
/// use std::path::Path;
/// use gibberish_or_not::download_model;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let path = Path::new("./my_model_dir");
///     
///     // Download with progress reporting
///     download_model(path, |progress| {
///         println!("Download progress: {:.1}%", progress * 100.0);
///     })?;
///     
///     println!("Model downloaded successfully!");
///     Ok(())
/// }
/// ```
pub fn download_model<P: AsRef<Path>>(path: P, progress: impl Fn(f32)) -> Result<(), ModelError> {
    let path = path.as_ref();
    fs::create_dir_all(path)?;
    
    let client = Client::builder()
        .timeout(Duration::from_secs(600)) // 10 minute timeout for large files
        .build()?;
    
    // Download each model file
    for (i, (filename, url)) in MODEL_FILES.iter().enumerate() {
        let file_path = path.join(filename);
        
        // Skip if file already exists
        if file_path.exists() {
            log::info!("File already exists, skipping: {}", filename);
            progress((i as f32 + 1.0) / MODEL_FILES.len() as f32);
            continue;
        }
        
        log::info!("Downloading: {} from {}", filename, url);
        
        // Start progress at the beginning of this file
        progress(i as f32 / MODEL_FILES.len() as f32);
        
        // Download file
        let mut response = client.get(*url).send()?;
        
        if !response.status().is_success() {
            return Err(ModelError::Model(
                format!("Failed to download {}: HTTP {}",
                    filename, response.status())
            ));
        }
        
        // Get content length if available
        let content_length = response.content_length().unwrap_or(0);
        
        // Create file
        let mut file = File::create(&file_path)?;
        
        // Copy data with progress reporting
        if content_length > 0 {
            let mut downloaded = 0;
            let mut buffer = [0; 8192];
            
            while let Ok(n) = response.read(&mut buffer) {
                if n == 0 {
                    break;
                }
                
                file.write_all(&buffer[..n])?;
                downloaded += n;
                
                // Calculate file progress
                let file_progress = downloaded as f64 / content_length as f64;
                
                // Calculate overall progress
                let overall_progress = (i as f32 + file_progress as f32) / MODEL_FILES.len() as f32;
                progress(overall_progress);
            }
        } else {
            // If content length is unknown, just copy without detailed progress
            copy(&mut response, &mut file)?;
            progress((i as f32 + 1.0) / MODEL_FILES.len() as f32);
        }
        
        log::info!("Downloaded: {}", filename);
    }
    
    // Create a model info file
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
///
/// Returns the default path where the model should be stored, which is typically
/// in the user's cache directory under "gibberish-or-not/model".
///
/// # Returns
///
/// A `PathBuf` pointing to the default model location
///
/// # Example
///
/// ```rust
/// use gibberish_or_not::default_model_path;
///
/// fn main() {
///     let model_path = default_model_path();
///     println!("Default model path: {}", model_path.display());
/// }
/// ```
pub fn default_model_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("./cache"))
        .join("gibberish-or-not")
        .join("model")
}

/// Check if the model exists at the specified path
///
/// This function checks if all required model files exist at the specified path.
///
/// # Arguments
///
/// * `path` - The path to check for model files
///
/// # Returns
///
/// `true` if all required model files exist, `false` otherwise
///
/// # Example
///
/// ```rust
/// use gibberish_or_not::{default_model_path, model_exists};
///
/// fn main() {
///     let path = default_model_path();
///     if model_exists(&path) {
///         println!("Model is available at: {}", path.display());
///     } else {
///         println!("Model is not available. Consider downloading it.");
///     }
/// }
/// ```
pub fn model_exists<P: AsRef<Path>>(path: P) -> bool {
    Model::exists(path.as_ref())
}

/// Download the model with a simple progress bar
///
/// This is a convenience function that downloads the model to the specified path
/// and displays a simple progress bar on the console. It's designed for CLI applications
/// or simple integration into Rust programs.
///
/// # Arguments
///
/// * `path` - The path where the model files should be downloaded
///
/// # Returns
///
/// * `Ok(())` if the download was successful
/// * `Err(ModelError)` if an error occurred during download
///
/// # Example
///
/// ```rust,no_run
/// use gibberish_or_not::{download_model_with_progress_bar, default_model_path};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Download to default path
///     download_model_with_progress_bar(default_model_path())?;
///     
///     // Or download to custom path
///     // download_model_with_progress_bar("./my_model_dir")?;
///     
///     println!("Model downloaded successfully!");
///     Ok(())
/// }
/// ```
pub fn download_model_with_progress_bar<P: AsRef<Path>>(path: P) -> Result<(), ModelError> {
    println!("Downloading model to: {}", path.as_ref().display());
    
    download_model(&path, |progress| {
        print!("\rDownload progress: {:.1}%", progress * 100.0);
        let _ = stdout().flush(); // Ignore flush errors
    })?;
    
    println!("\nModel downloaded successfully to: {}", path.as_ref().display());
    Ok(())
}