use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
    pub table_cell_matching: bool,
    pub include_images: bool,
    pub image_scale: f64,
    pub image_export_mode: String,
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
    pub to_formats: Vec<String>,
    pub from_formats: Vec<String>,
    pub page_range: Option<Vec<i64>>,
    pub md_page_break_placeholder: String,
    pub output_dir: Option<String>,
}

impl Default for DoclingServeConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:5001".to_string(),
            api_key: std::env::var("DOCLING_SERVE_API_KEY").ok(),
            use_ocr: false,
            force_ocr: false,
            ocr_engine: "easyocr".to_string(),
            ocr_languages: vec![],
            pdf_backend: "dlparse_v4".to_string(),
            table_mode: "accurate".to_string(),
            enable_table_structure: true,
            table_cell_matching: true,
            include_images: false,
            image_scale: 2.0,
            image_export_mode: "embedded".to_string(),
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
            to_formats: vec!["md".to_string()],
            from_formats: vec![
                "docx".to_string(),
                "pptx".to_string(),
                "html".to_string(),
                "image".to_string(),
                "pdf".to_string(),
                "asciidoc".to_string(),
                "md".to_string(),
            ],
            page_range: None,
            md_page_break_placeholder: "".to_string(),
            output_dir: None, // Use default output directory (~/.parse)
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
        form_data.push(("do_ocr", self.use_ocr.to_string()));
        form_data.push(("force_ocr", self.force_ocr.to_string()));
        form_data.push(("do_table_structure", self.enable_table_structure.to_string()));
        form_data.push(("table_cell_matching", self.table_cell_matching.to_string()));
        form_data.push(("include_images", self.include_images.to_string()));
        form_data.push(("do_code_enrichment", self.do_code_enrichment.to_string()));
        form_data.push(("do_formula_enrichment", self.do_formula_enrichment.to_string()));
        form_data.push(("do_picture_classification", self.do_picture_classification.to_string()));
        form_data.push(("do_picture_description", self.do_picture_description.to_string()));
        form_data.push(("abort_on_error", self.abort_on_error.to_string()));

        // Add string parameters
        form_data.push(("ocr_engine", self.ocr_engine.clone()));
        form_data.push(("pdf_backend", self.pdf_backend.clone()));
        form_data.push(("table_mode", self.table_mode.clone()));
        form_data.push(("pipeline", self.processing_pipeline.clone()));
        form_data.push(("image_export_mode", self.image_export_mode.clone()));

        if !self.md_page_break_placeholder.is_empty() {
            form_data.push(("md_page_break_placeholder", self.md_page_break_placeholder.clone()));
        }

        // Add numeric parameters
        form_data.push(("images_scale", self.image_scale.to_string()));
        form_data.push(("document_timeout", self.document_timeout.to_string()));

        // Add array parameters by repeating the field name for each value
        // This is the standard way to send arrays in multipart/form-data
        // Example: to_formats=md&to_formats=json&to_formats=html

        if !self.ocr_languages.is_empty() {
            for lang in &self.ocr_languages {
                form_data.push(("ocr_lang", lang.clone()));
            }
        }

        if !self.to_formats.is_empty() {
            for format in &self.to_formats {
                form_data.push(("to_formats", format.clone()));
            }
        }

        if !self.from_formats.is_empty() {
            for format in &self.from_formats {
                form_data.push(("from_formats", format.clone()));
            }
        }

        if let Some(ref page_range) = self.page_range {
            // page_range is special - it's an array of exactly 2 numbers
            if page_range.len() == 2 {
                form_data.push(("page_range", page_range[0].to_string()));
                form_data.push(("page_range", page_range[1].to_string()));
            }
        }

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