use clap::Parser;
use gibberish_or_not::{
    check_token_status, default_model_path, download_model_with_progress_bar, model_exists,
    ModelError, TokenStatus,
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Download the enhanced gibberish detection model"
)]
struct Args {
    /// Path to download model to (default: system cache directory)
    #[arg(short, long)]
    model_path: Option<String>,

    /// Force download even if model already exists
    #[arg(short, long)]
    force: bool,
}
