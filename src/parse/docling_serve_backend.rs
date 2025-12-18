use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use serde::Deserialize;

use crate::parse::cache::CacheManager;
use crate::parse::docling_serve_config::DoclingServeConfig;
use crate::parse::error::JobError;


#[derive(Debug, Deserialize)]
struct TaskStatusResponse {
    task_id: String,
    status: String,
    result: Option<serde_json::Value>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HealthCheckResponse {
    status: String,
}

pub struct DoclingServeBackend {
    config: DoclingServeConfig,
    cache_manager: CacheManager,
    client: reqwest::Client,
    verbose: bool,
}

impl DoclingServeBackend {
    pub fn new(config: DoclingServeConfig, verbose: bool) -> anyhow::Result<Self> {
        let cache_dir = if let Some(ref custom_output_dir) = config.output_dir {
            std::path::PathBuf::from(custom_output_dir)
        } else {
            dirs::home_dir()
                .ok_or_else(|| anyhow::Error::msg("Could not find home directory"))?
                .join(".parse")
        };

        fs::create_dir_all(&cache_dir)?;

        // Create HTTP client with timeout
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.document_timeout as u64 + 30)) // Add buffer
            .build()?;

        Ok(Self {
            config,
            cache_manager: CacheManager::new(cache_dir),
            client,
            verbose,
        })
    }

    pub async fn parse(&self, files: Vec<String>) -> Result<Vec<String>, JobError> {
        // Check if docling-serve is available
        self.check_service_availability().await?;

        let semaphore = Arc::new(Semaphore::new(10)); // Reasonable concurrency limit
        let mut handles = Vec::new();
        let mut results = Vec::new();

        for file_path in files {
            // Skip if file doesn't need parsing (already text-based)
            if self.cache_manager.should_skip_file(&file_path) {
                if self.verbose {
                    eprintln!("Skipping readable file: {file_path}");
                }
                results.push(file_path);
                continue;
            }

            // Check cache first
            if let Ok(cached_path) = self.cache_manager.get_cached_result(&file_path).await {
                if self.verbose {
                    eprintln!("Using cached result for: {file_path}");
                }
                results.push(cached_path);
                continue;
            }

            let semaphore = Arc::clone(&semaphore);
            let config = self.config.clone();
            let cache_manager = CacheManager::new(self.cache_manager.cache_dir.clone());
            let client = self.client.clone();
            let verbose = self.verbose;

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire_owned().await.unwrap();

                Self::process_single_document(
                    client,
                    file_path,
                    config,
                    cache_manager,
                    verbose,
                )
                .await
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let result = handle.await?;
            match result {
                Ok(path) => results.push(path),
                Err(e) => eprintln!("Error processing file: {e:?}"),
            }
        }

        Ok(results)
    }

    async fn process_single_document(
        client: reqwest::Client,
        file_path: String,
        config: DoclingServeConfig,
        cache_manager: CacheManager,
        verbose: bool,
    ) -> Result<String, JobError> {
        if verbose {
            eprintln!("Processing file with docling-serve: {file_path}");
        }

        // Verify input file exists
        if !Path::new(&file_path).exists() {
            return Err(JobError::InvalidResponse(format!(
                "File not found: {}",
                file_path
            )));
        }

        let response_data = if config.use_async {
            Self::process_async(&client, &file_path, &config, verbose).await?
        } else {
            Self::process_sync(&client, &file_path, &config, verbose).await?
        };

        // Extract and write all content types from response
        Self::write_content_files(&cache_manager, &file_path, response_data).await
    }

    async fn process_sync(
        client: &reqwest::Client,
        file_path: &str,
        config: &DoclingServeConfig,
        verbose: bool,
    ) -> Result<serde_json::Value, JobError> {
        // Read the file
        let file_content = fs::read(file_path).map_err(|e| {
            JobError::InvalidResponse(format!("Failed to read input file: {}", e))
        })?;

        let filename = Path::new(file_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document");

        if verbose {
            eprintln!("Uploading file to docling-serve: {}", config.get_convert_endpoint());
        }

        // Create multipart form
        let mut form = reqwest::multipart::Form::new();
        
        // Add file
        let file_part = reqwest::multipart::Part::bytes(file_content)
            .file_name(filename.to_string());
        form = form.part("files", file_part);

        // Add form parameters
        for (key, value) in config.build_form_data() {
            form = form.text(key, value);
        }

        // Send request
        let mut request = client
            .post(config.get_convert_endpoint())
            .multipart(form);

        // Add API key if provided
        if let Some(ref api_key) = config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await.map_err(|e| {
            JobError::InvalidResponse(format!("Failed to send request to docling-serve: {}", e))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(JobError::InvalidResponse(format!(
                "Docling-serve returned error {}: {}",
                status,
                error_text
            )));
        }

        // Parse JSON response
        let response_data: serde_json::Value = response.json().await.map_err(|e| {
            JobError::InvalidResponse(format!("Failed to parse docling-serve response: {}", e))
        })?;

        Ok(response_data)
    }

    async fn process_async(
        client: &reqwest::Client,
        file_path: &str,
        config: &DoclingServeConfig,
        verbose: bool,
    ) -> Result<serde_json::Value, JobError> {
        // Submit async task
        let task_id = Self::submit_async_task(client, file_path, config, verbose).await?;

        if verbose {
            eprintln!("Submitted async task with ID: {}", task_id);
        }

        // Poll for completion
        Self::poll_task_completion(client, &task_id, config, verbose).await
    }

    async fn submit_async_task(
        client: &reqwest::Client,
        file_path: &str,
        config: &DoclingServeConfig,
        verbose: bool,
    ) -> Result<String, JobError> {
        // Read the file
        let file_content = fs::read(file_path).map_err(|e| {
            JobError::InvalidResponse(format!("Failed to read input file: {}", e))
        })?;

        let filename = Path::new(file_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document");

        if verbose {
            eprintln!("Submitting async task to: {}", config.get_convert_endpoint());
        }

        // Create multipart form  
        let mut form = reqwest::multipart::Form::new();
        
        // Add file
        let file_part = reqwest::multipart::Part::bytes(file_content)
            .file_name(filename.to_string());
        form = form.part("files", file_part);

        // Add form parameters
        for (key, value) in config.build_form_data() {
            form = form.text(key, value);
        }

        // Send request
        let mut request = client
            .post(config.get_convert_endpoint())
            .multipart(form);

        // Add API key if provided
        if let Some(ref api_key) = config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await.map_err(|e| {
            JobError::InvalidResponse(format!("Failed to send async request: {}", e))
        })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(JobError::InvalidResponse(format!(
                "Failed to submit async task: {}",
                error_text
            )));
        }

        let task_response: TaskStatusResponse = response.json().await.map_err(|e| {
            JobError::InvalidResponse(format!("Failed to parse task response: {}", e))
        })?;

        Ok(task_response.task_id)
    }

    async fn poll_task_completion(
        client: &reqwest::Client,
        task_id: &str,
        config: &DoclingServeConfig,
        verbose: bool,
    ) -> Result<serde_json::Value, JobError> {
        let mut attempts = 0;

        while attempts < config.max_poll_attempts {
            attempts += 1;

            if verbose && attempts % 5 == 0 {
                eprintln!("Polling task {} (attempt {})", task_id, attempts);
            }

            // Check task status
            let mut request = client.get(config.get_status_endpoint(task_id));
            
            if let Some(ref api_key) = config.api_key {
                request = request.header("Authorization", format!("Bearer {}", api_key));
            }

            let response = request.send().await.map_err(|e| {
                JobError::InvalidResponse(format!("Failed to poll task status: {}", e))
            })?;

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(JobError::InvalidResponse(format!(
                    "Task status polling failed: {}",
                    error_text
                )));
            }

            let status: TaskStatusResponse = response.json().await.map_err(|e| {
                JobError::InvalidResponse(format!("Failed to parse status response: {}", e))
            })?;

            match status.status.as_str() {
                "completed" => {
                    // Get the result
                    return Self::get_task_result(client, task_id, config).await;
                }
                "failed" => {
                    let error_msg = status.error.unwrap_or("Unknown error".to_string());
                    return Err(JobError::InvalidResponse(format!(
                        "Task failed: {}",
                        error_msg
                    )));
                }
                "pending" | "running" => {
                    // Continue polling
                    tokio::time::sleep(Duration::from_secs(config.poll_interval)).await;
                }
                _ => {
                    return Err(JobError::InvalidResponse(format!(
                        "Unknown task status: {}",
                        status.status
                    )));
                }
            }
        }

        Err(JobError::TimeoutError)
    }

    async fn get_task_result(
        client: &reqwest::Client,
        task_id: &str,
        config: &DoclingServeConfig,
    ) -> Result<serde_json::Value, JobError> {
        let mut request = client.get(config.get_result_endpoint(task_id));
        
        if let Some(ref api_key) = config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await.map_err(|e| {
            JobError::InvalidResponse(format!("Failed to get task result: {}", e))
        })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(JobError::InvalidResponse(format!(
                "Failed to get task result: {}",
                error_text
            )));
        }

        let result: serde_json::Value = response.json().await.map_err(|e| {
            JobError::InvalidResponse(format!("Failed to parse result: {}", e))
        })?;

        Ok(result)
    }

    async fn write_content_files(
        cache_manager: &CacheManager,
        file_path: &str,
        response: serde_json::Value,
    ) -> Result<String, JobError> {
        use std::path::Path;
        
        let path = Path::new(file_path);
        let filename = path.file_name().unwrap().to_str().unwrap();
        let mut created_files = Vec::new();
        let mut primary_output = None;


        // Extract content from ConvertDocumentResponse structure
        // The actual structure is: { "document": { "md_content": "...", "html_content": "..." }, ... }
        if let Some(document) = response.get("document") {
            Self::extract_content_from_document(cache_manager, filename, document, &mut created_files, &mut primary_output)?;
        } else {
            // Fallback: try documents array (for compatibility)
            if let Some(documents) = response.get("documents").and_then(|d| d.as_array()) {
                if let Some(first_doc) = documents.first() {
                    Self::extract_content_from_document(cache_manager, filename, first_doc, &mut created_files, &mut primary_output)?;
                }
            }
            // Last resort: try the response itself as a document
            else {
                Self::extract_content_from_document(cache_manager, filename, &response, &mut created_files, &mut primary_output)?;
            }
        }

        // If no content was found, create a JSON file with the full response
        if created_files.is_empty() {
            let json_content = serde_json::to_string_pretty(&response).map_err(|e| {
                JobError::InvalidResponse(format!("Failed to serialize response: {}", e))
            })?;
            
            let output_path = cache_manager.cache_dir.join(format!("{}.json", filename));
            fs::write(&output_path, json_content).map_err(|e| {
                JobError::InvalidResponse(format!("Failed to write fallback JSON file: {}", e))
            })?;

            let output_path_str = output_path.to_string_lossy().to_string();
            created_files.push(output_path_str.clone());
            primary_output = Some(output_path_str);
        }

        // Write metadata for the primary output file
        if let Some(ref primary_path) = primary_output {
            Self::write_metadata(cache_manager, file_path, primary_path)?;
        }

        // Return the primary output path (preferably markdown, otherwise the first created file)
        primary_output.ok_or_else(|| {
            JobError::InvalidResponse("No content could be extracted from response".to_string())
        })
    }

    fn extract_content_from_document(
        cache_manager: &CacheManager,
        filename: &str,
        document: &serde_json::Value,
        created_files: &mut Vec<String>,
        primary_output: &mut Option<String>,
    ) -> Result<(), JobError> {
        
        // Extract different content types
        let string_content_types = [
            ("md_content", "md"),
            ("html_content", "html"), 
            ("text_content", "txt"),
            ("doctags_content", "doctags"),
            // Fallback field names for compatibility
            ("markdown", "md"),
            ("html", "html"),
            ("text", "txt"),
        ];

        // Handle string content types
        for (field_name, extension) in string_content_types {
            if let Some(content) = document.get(field_name).and_then(|c| c.as_str()) {
                if !content.trim().is_empty() {
                    let output_path = cache_manager.cache_dir.join(format!("{}.{}", filename, extension));
                    
                    fs::write(&output_path, content).map_err(|e| {
                        JobError::InvalidResponse(format!("Failed to write {} file: {}", extension, e))
                    })?;

                    let output_path_str = output_path.to_string_lossy().to_string();
                    created_files.push(output_path_str.clone());
                    
                    // Set markdown as primary, or first content type found
                    if extension == "md" || primary_output.is_none() {
                        *primary_output = Some(output_path_str);
                    }
                }
            }
        }

        // Handle json_content separately since it's an object, not a string
        if let Some(json_content) = document.get("json_content") {
            if !json_content.is_null() {
                let json_str = serde_json::to_string_pretty(&json_content).map_err(|e| {
                    JobError::InvalidResponse(format!("Failed to serialize json_content: {}", e))
                })?;
                
                let output_path = cache_manager.cache_dir.join(format!("{}.json", filename));
                fs::write(&output_path, json_str).map_err(|e| {
                    JobError::InvalidResponse(format!("Failed to write json file: {}", e))
                })?;

                let output_path_str = output_path.to_string_lossy().to_string();
                created_files.push(output_path_str.clone());
                
                // Set json as output if no markdown found
                if primary_output.is_none() {
                    *primary_output = Some(output_path_str);
                }
            }
        }
        Ok(())
    }

    fn write_metadata(
        cache_manager: &CacheManager,
        original_file_path: &str,
        output_file_path: &str,
    ) -> Result<(), JobError> {
        use std::time::UNIX_EPOCH;
        use crate::parse::cache::FileMetadata;

        let path = Path::new(original_file_path);
        let filename = path.file_name().unwrap().to_str().unwrap();
        
        // Write metadata
        let metadata_path = cache_manager.cache_dir.join(format!("{}.metadata.json", filename));
        let file_metadata = fs::metadata(path)?;

        let modified_time = file_metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let metadata = FileMetadata {
            modified_time,
            size: file_metadata.len(),
            parsed_path: output_file_path.to_string(),
        };

        fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;
        Ok(())
    }

    async fn check_service_availability(&self) -> Result<(), JobError> {
        if self.verbose {
            eprintln!("Checking docling-serve availability at: {}", self.config.get_health_endpoint());
        }

        let mut request = self.client.get(self.config.get_health_endpoint());
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await.map_err(|_| {
            JobError::InvalidResponse(format!(
                "Docling-serve is not available at {}. Please start the service or check the URL.",
                self.config.base_url
            ))
        })?;

        if !response.status().is_success() {
            return Err(JobError::InvalidResponse(format!(
                "Docling-serve health check failed with status: {}",
                response.status()
            )));
        }

        // Try to parse health response (consume response but don't print status)
        let _ = response.json::<HealthCheckResponse>().await;

        Ok(())
    }
}