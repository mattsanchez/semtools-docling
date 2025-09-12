use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoclingConfig {
    pub use_ocr: bool,
    pub vlm_model: Option<String>,
    pub output_format: String,
    pub python_path: Option<String>,
    pub extra_args: Vec<String>,
    pub enable_tables: bool,
    pub enable_images: bool,
    pub cache_dir: Option<String>,
}

impl Default for DoclingConfig {
    fn default() -> Self {
        Self {
            use_ocr: true,
            vlm_model: None, // Can be "smoldocling" or other VLM models
            output_format: "md".to_string(), // Default to markdown
            python_path: None, // Use system Python by default
            extra_args: Vec::new(),
            enable_tables: true,
            enable_images: true,
            cache_dir: None, // Use default cache directory
        }
    }
}

impl DoclingConfig {
    pub fn from_config_file(path: &str) -> anyhow::Result<Self> {
        if !Path::new(path).exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)?;
        let config: DoclingConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    /// Build command line arguments for the docling CLI
    pub fn build_cli_args(&self, input_file: &str, output_dir: &str) -> Vec<String> {
        let mut args = vec![input_file.to_string()];

        // Add output format
        args.push("--to".to_string());
        args.push(self.output_format.clone());

        // Add output directory
        args.push("--output".to_string());
        args.push(output_dir.to_string());

        // OCR settings
        if !self.use_ocr {
            args.push("--no-ocr".to_string());
        }

        // VLM model if specified
        if let Some(ref model) = self.vlm_model {
            args.push("--pipeline".to_string());
            args.push("vlm".to_string());
            args.push("--vlm-model".to_string());
            args.push(model.clone());
        }

        // Add any extra arguments
        args.extend(self.extra_args.clone());

        args
    }

    /// Get the Python executable path
    pub fn get_python_path(&self) -> String {
        self.python_path
            .clone()
            .unwrap_or_else(|| "python3".to_string())
    }
}
