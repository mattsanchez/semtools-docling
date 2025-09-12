use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoclingServeConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub use_ocr: bool,
    pub force_ocr: bool,
    pub ocr_engine: String,
    pub ocr_languages: Vec<String>,
    pub pdf_backend: String,
    pub table_mode: String,
    pub enable_table_structure: bool,
    pub enable_images: bool,
    pub image_scale: f64,
    pub do_code_enrichment: bool,
    pub do_formula_enrichment: bool,
    pub do_picture_classification: bool,
    pub do_picture_description: bool,
    pub processing_pipeline: String,
    pub document_timeout: f64,
    pub abort_on_error: bool,
    pub use_async: bool,
    pub poll_interval: u64,
    pub max_poll_attempts: usize,
}

impl Default for DoclingServeConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:5001".to_string(),
            api_key: std::env::var("DOCLING_SERVE_API_KEY").ok(),
            use_ocr: true,
            force_ocr: false,
            ocr_engine: "easyocr".to_string(),
            ocr_languages: vec![],
            pdf_backend: "dlparse_v4".to_string(),
            table_mode: "accurate".to_string(),
            enable_table_structure: true,
            enable_images: true,
            image_scale: 2.0,
            do_code_enrichment: false,
            do_formula_enrichment: false,
            do_picture_classification: false,
            do_picture_description: false,
            processing_pipeline: "standard".to_string(),
            document_timeout: 604800.0, // 7 days
            abort_on_error: false,
            use_async: false, // Use synchronous API by default
            poll_interval: 5, // seconds
            max_poll_attempts: 60, // 5 minutes total
        }
    }
}

impl DoclingServeConfig {
    pub fn from_config_file(path: &str) -> anyhow::Result<Self> {
        if !Path::new(path).exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)?;
        let config: DoclingServeConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    /// Build the request body for file conversion
    pub fn build_form_data(&self) -> Vec<(&'static str, String)> {
        let mut form_data = vec![];

        // Add boolean parameters
        form_data.push(("convert_do_ocr", self.use_ocr.to_string()));
        form_data.push(("convert_force_ocr", self.force_ocr.to_string()));
        form_data.push(("convert_do_table_structure", self.enable_table_structure.to_string()));
        form_data.push(("convert_include_images", self.enable_images.to_string()));
        form_data.push(("convert_do_code_enrichment", self.do_code_enrichment.to_string()));
        form_data.push(("convert_do_formula_enrichment", self.do_formula_enrichment.to_string()));
        form_data.push(("convert_do_picture_classification", self.do_picture_classification.to_string()));
        form_data.push(("convert_do_picture_description", self.do_picture_description.to_string()));
        form_data.push(("convert_abort_on_error", self.abort_on_error.to_string()));

        // Add string parameters
        form_data.push(("convert_ocr_engine", self.ocr_engine.clone()));
        form_data.push(("convert_pdf_backend", self.pdf_backend.clone()));
        form_data.push(("convert_table_mode", self.table_mode.clone()));
        form_data.push(("convert_pipeline", self.processing_pipeline.clone()));

        // Add numeric parameters
        form_data.push(("convert_images_scale", self.image_scale.to_string()));
        form_data.push(("convert_document_timeout", self.document_timeout.to_string()));

        form_data
    }

    /// Get the conversion endpoint URL
    pub fn get_convert_endpoint(&self) -> String {
        if self.use_async {
            format!("{}/v1/convert/file/async", self.base_url)
        } else {
            format!("{}/v1/convert/file", self.base_url)
        }
    }

    /// Get the task status endpoint URL
    pub fn get_status_endpoint(&self, task_id: &str) -> String {
        format!("{}/v1/status/poll/{}", self.base_url, task_id)
    }

    /// Get the task result endpoint URL
    pub fn get_result_endpoint(&self, task_id: &str) -> String {
        format!("{}/v1/result/{}", self.base_url, task_id)
    }

    /// Get the health check endpoint URL
    pub fn get_health_endpoint(&self) -> String {
        format!("{}/health", self.base_url)
    }
}