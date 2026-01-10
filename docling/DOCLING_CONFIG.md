# Docling Server Configuration Guide

This guide explains how to configure semtools to use the Docling Server backend for document parsing.

## Quick Start

1. **Create a minimal configuration** (all fields are optional):
   ```bash
   cat > ~/.semtools_config.json << 'EOF'
   {
     "docling_serve": {
       "base_url": "http://localhost:15001"
     }
   }
   EOF
   ```

2. **Use the parse tool** with the docling-serve backend (default):
   ```bash
   parse document.pdf
   ```

**Note:** All configuration fields are optional. Any field you don't specify will use its default value. You only need to set the fields you want to customize.

## Configuration Options

**Important:** All configuration fields are optional and will use sensible defaults if not specified. You only need to configure the settings you want to change.

### Connection Settings

- **`base_url`** (string)
  - URL of your Docling Server instance
  - Default: `"http://localhost:5001"`
  - Example: `"http://localhost:15001"`

- **`api_key`** (string, optional)
  - API key for authentication if your Docling Server requires it
  - Can also be set via `DOCLING_SERVE_API_KEY` environment variable
  - Default: `null`

### OCR Settings

- **`use_ocr`** (boolean)
  - Enable OCR for document processing
  - Default: `true`

- **`force_ocr`** (boolean)
  - Force OCR even for documents with extractable text
  - Default: `false`

- **`ocr_engine`** (string)
  - OCR engine to use
  - Options: `"auto"`, `"easyocr"`, `"ocrmac"`, `"rapidocr"`, `"tesserocr"`, `"tesseract"`
  - Default: `"easyocr"`

- **`ocr_languages`** (array of strings)
  - Languages for OCR processing
  - Example: `["eng", "spa", "fra"]`
  - Default: `[]` (auto-detect)
  - Note: Language codes vary by OCR engine

### PDF Processing

- **`pdf_backend`** (string)
  - PDF parsing backend to use
  - Options: `"pypdfium2"`, `"dlparse_v1"`, `"dlparse_v2"`, `"dlparse_v4"`
  - Default: `"dlparse_v4"`

### Table Extraction

- **`table_mode`** (string)
  - Table extraction mode
  - Options: `"accurate"`, `"fast"`
  - Default: `"accurate"`

- **`enable_table_structure`** (boolean)
  - Extract table structure information
  - Default: `true`

- **`table_cell_matching`** (boolean)
  - Match table cell predictions back to PDF cells
  - Can break table output in edge cases but improves accuracy
  - Default: `true`

### Image Processing

- **`enable_images`** (boolean)
  - Include images in the output
  - Default: `true`

- **`image_scale`** (number)
  - Scale factor for image extraction
  - Default: `2.0`

- **`image_export_mode`** (string)
  - Image export mode for documents
  - Options: `"placeholder"`, `"embedded"`, `"referenced"`
  - Default: `"embedded"`

### Content Enrichment

- **`do_code_enrichment`** (boolean)
  - Enhance code block detection and formatting
  - Default: `false`

- **`do_formula_enrichment`** (boolean)
  - Enhance mathematical formula detection
  - Default: `false`

- **`do_picture_classification`** (boolean)
  - Classify images by type
  - Default: `false`

- **`do_picture_description`** (boolean)
  - Generate descriptions for images
  - Default: `false`

### Processing Pipeline

- **`processing_pipeline`** (string)
  - Pipeline to use for document processing
  - Options: `"standard"`, `"simple"`, `"advanced"`
  - Default: `"standard"`

### Output Formats and Filters

- **`to_formats`** (array of strings)
  - Output format(s) to generate
  - Options: `"md"`, `"json"`, `"html"`, `"html_split_page"`, `"text"`, `"doctags"`
  - Default: `["md"]`
  - Example: `["md", "json", "html"]` to generate all three formats

- **`from_formats`** (array of strings)
  - Input format(s) to accept for processing
  - Options: `"docx"`, `"pptx"`, `"html"`, `"image"`, `"pdf"`, `"asciidoc"`, `"md"`, `"csv"`, `"xlsx"`, `"xml_uspto"`, `"xml_jats"`, `"mets_gbs"`, `"json_docling"`, `"audio"`, `"vtt"`
  - Default: Common formats (`["docx", "pptx", "html", "image", "pdf", "asciidoc", "md"]`)
  - Set to limit which file types are processed

- **`page_range`** (array of two numbers, optional)
  - Only process a specific range of pages
  - Format: `[start_page, end_page]` (1-indexed)
  - Default: `null` (process all pages)
  - Example: `[1, 10]` to process only first 10 pages

- **`md_page_break_placeholder`** (string)
  - Placeholder text to insert between pages in markdown output
  - Default: `""` (empty, no page breaks)
  - Example: `"\\n---\\n"` for a horizontal rule between pages

### Async Processing

- **`use_async`** (boolean)
  - Use asynchronous API for long-running documents
  - When `true`, documents are processed asynchronously and polled for results
  - Default: `false` (synchronous processing)

- **`poll_interval`** (number)
  - Seconds between status checks for async jobs
  - Only used when `use_async` is `true`
  - Default: `5`

- **`max_poll_attempts`** (number)
  - Maximum number of polling attempts before timeout
  - Only used when `use_async` is `true`
  - Default: `60` (5 minutes at 5-second intervals)

### Timeouts and Error Handling

- **`document_timeout`** (number)
  - Maximum processing time in seconds
  - Default: `604800.0` (7 days)

- **`abort_on_error`** (boolean)
  - Abort processing if an error occurs
  - Default: `false`

## Example Configurations

### Minimal Configuration (Custom Port)
```json
{
  "docling_serve": {
    "base_url": "http://localhost:15001"
  }
}
```

### High-Quality Document Processing
```json
{
  "docling_serve": {
    "base_url": "http://localhost:15001",
    "use_ocr": true,
    "force_ocr": false,
    "ocr_engine": "easyocr",
    "pdf_backend": "dlparse_v4",
    "table_mode": "accurate",
    "enable_table_structure": true,
    "table_cell_matching": true,
    "enable_images": true,
    "image_scale": 2.0,
    "image_export_mode": "embedded",
    "do_formula_enrichment": true,
    "do_code_enrichment": true,
    "to_formats": ["md", "json", "html"]
  }
}
```

### Fast Processing (Lower Quality)
```json
{
  "docling_serve": {
    "base_url": "http://localhost:15001",
    "use_ocr": false,
    "pdf_backend": "pypdfium2",
    "table_mode": "fast",
    "enable_table_structure": false,
    "table_cell_matching": false,
    "enable_images": false,
    "image_export_mode": "none",
    "do_code_enrichment": false,
    "do_formula_enrichment": false,
    "to_formats": ["md"]
  }
}
```

### Async Processing for Large Documents
```json
{
  "docling_serve": {
    "base_url": "http://localhost:15001",
    "use_async": true,
    "poll_interval": 10,
    "max_poll_attempts": 120,
    "document_timeout": 3600.0
  }
}
```

### Multilingual OCR
```json
{
  "docling_serve": {
    "base_url": "http://localhost:15001",
    "use_ocr": true,
    "force_ocr": true,
    "ocr_engine": "easyocr",
    "ocr_languages": ["eng", "spa", "fra", "deu", "ita"]
  }
}
```

### Process Specific Pages Only
```json
{
  "docling_serve": {
    "base_url": "http://localhost:15001",
    "page_range": [1, 10]
  }
}
```

### Generate Multiple Output Formats
```json
{
  "docling_serve": {
    "base_url": "http://localhost:15001",
    "to_formats": ["md", "json", "html", "text"],
    "md_page_break_placeholder": "\n\n---\n\n"
  }
}
```

## Usage

### Using the Default Backend
By default, semtools uses the docling-serve backend:
```bash
parse document.pdf
```

### Explicitly Specifying the Backend
```bash
parse --backend docling-serve document.pdf
```

### Using a Custom Config File
```bash
parse --config /path/to/custom_config.json document.pdf
```

### Verbose Output
```bash
parse --verbose document.pdf
```

### Processing Multiple Files
```bash
parse document1.pdf document2.docx document3.pdf
```

## Starting Docling Server

If you need to start a Docling Server instance on port 15001:

```bash
# Using Docker
docker run -p 15001:5001 docling/docling-serve

# Or if running locally
docling-serve --port 15001
```

## Backend Comparison

semtools supports multiple parsing backends:

- **`docling-serve`** (default): Server-based parsing with advanced features
- **`docling`**: Local Docling library (Python-based)
- **`llama-parse`**: LlamaIndex cloud-based parsing service

To switch backends, use the `--backend` flag or configure the appropriate section in your config file.
