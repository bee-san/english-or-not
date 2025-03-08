use std::io::{self, Write};
use std::path::Path;

use crate::model::{download_model, ModelError};

/// Function for CLI tool to download model with progress bar
pub fn download_with_progress_bar<P: AsRef<Path>>(path: P) -> Result<(), ModelError> {
    download_model(
        &path,
        |progress| {
            print!("\rDownload progress: {:.1}%", progress * 100.0);
            let _ = io::stdout().flush();
        },
        None,
    )?;

    println!("\nDownload complete");
    Ok(())
}
