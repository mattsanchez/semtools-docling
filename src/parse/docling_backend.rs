use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Semaphore;

use crate::parse::cache::CacheManager;
use crate::parse::docling_config::DoclingConfig;
use crate::parse::error::JobError;

pub struct DoclingBackend {
    config: DoclingConfig,
    cache_manager: CacheManager,
    verbose: bool,
}

impl DoclingBackend {
    pub fn new(config: DoclingConfig, verbose: bool) -> anyhow::Result<Self> {
        let cache_dir = if let Some(ref custom_output_dir) = config.output_dir {
            std::path::PathBuf::from(custom_output_dir)
        } else {
            dirs::home_dir()
                .ok_or_else(|| anyhow::Error::msg("Could not find home directory"))?
                .join(".parse")
        };

        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            config,
            cache_manager: CacheManager::new(cache_dir),
            verbose,
        })
    }

    pub async fn parse(&self, files: Vec<String>) -> Result<Vec<String>, JobError> {
        // Check if docling is available
        self.check_docling_availability().await?;

        let semaphore = Arc::new(Semaphore::new(10)); // Use a reasonable concurrency limit
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
            let verbose = self.verbose;

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire_owned().await.unwrap();

                Self::process_single_document(file_path, config, cache_manager, verbose).await
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
        file_path: String,
        config: DoclingConfig,
        cache_manager: CacheManager,
        verbose: bool,
    ) -> Result<String, JobError> {
        if verbose {
            eprintln!("Processing file with Docling: {file_path}");
        }

        // Verify input file exists
        if !Path::new(&file_path).exists() {
            return Err(JobError::InvalidResponse(format!(
                "File not found: {}",
                file_path
            )));
        }

        // Create temporary output directory
        let temp_dir = std::env::temp_dir().join(format!(
            "docling_{}",
            std::process::id() as u64 * 1000000
                + std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64
                    % 1000000
        ));
        fs::create_dir_all(&temp_dir).map_err(|e| {
            JobError::InvalidResponse(format!("Failed to create temp directory: {}", e))
        })?;

        // Build docling command
        let args = config.build_cli_args(&file_path, temp_dir.to_string_lossy().as_ref());

        if verbose {
            eprintln!("Running: docling {}", args.join(" "));
        }

        // Execute docling command
        let output = Command::new("docling")
            .args(&args)
            .output()
            .await
            .map_err(|e| JobError::InvalidResponse(format!("Failed to execute docling: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JobError::InvalidResponse(format!(
                "Docling command failed: {}",
                stderr
            )));
        }

        // Find the generated output file in the temporary directory
        let input_filename = Path::new(&file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let expected_output_file =
            temp_dir.join(format!("{}.{}", input_filename, config.output_format));

        let content = if expected_output_file.exists() {
            fs::read_to_string(&expected_output_file).map_err(|e| {
                JobError::InvalidResponse(format!("Failed to read docling output: {}", e))
            })?
        } else {
            // If the expected file doesn't exist, try to find any file with the right extension
            let entries = fs::read_dir(&temp_dir).map_err(|e| {
                JobError::InvalidResponse(format!("Failed to read temp directory: {}", e))
            })?;

            let mut found_file = None;
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == config.output_format.as_str() {
                        found_file = Some(path);
                        break;
                    }
                }
            }

            if let Some(output_file) = found_file {
                fs::read_to_string(&output_file).map_err(|e| {
                    JobError::InvalidResponse(format!("Failed to read docling output: {}", e))
                })?
            } else {
                return Err(JobError::InvalidResponse(
                    "No output file found after docling conversion".to_string(),
                ));
            }
        };

        // Clean up temporary directory
        let _ = fs::remove_dir_all(&temp_dir);

        // Write results to cache
        cache_manager
            .write_results_to_disk(&file_path, &content)
            .await
    }

    async fn check_docling_availability(&self) -> Result<(), JobError> {
        let output = Command::new("docling")
            .arg("--version")
            .output()
            .await
            .map_err(|_| {
                JobError::InvalidResponse(
                    "Docling is not installed or not available in PATH. Please install with: pip install docling".to_string()
                )
            })?;

        if !output.status.success() {
            return Err(JobError::InvalidResponse(
                "Docling is not working properly. Please check your installation.".to_string(),
            ));
        }

        Ok(())
    }
}
