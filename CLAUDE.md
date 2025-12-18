# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SemTools is a high-performance CLI tool suite for document processing and semantic search, built with Rust. It provides two main tools:
- **`parse`** - Document parsing (PDF, DOCX, etc.) using LlamaParse API with caching
- **`search`** - Local semantic search using multilingual embeddings with cosine similarity

The project uses a hybrid Rust/NPM distribution model: core functionality in Rust, distributed via both Cargo and NPM.

## Development Commands

### Building and Testing
```bash
# Build the project
cargo build

# Run tests
cargo test

# Run linting and formatting
cargo clippy
cargo fmt

# Install for development
cargo install --path .
```

### Feature-Specific Builds
```bash
# Install only parse tool
cargo install semtools --no-default-features --features=parse

# Install only search tool  
cargo install semtools --no-default-features --features=search
```

### NPM Distribution
```bash
# Install dependencies and build
npm install
```

## Architecture

### Core Structure
- **Dual binary system**: Two separate CLI tools (`parse` and `search`) with optional features
- **src/bin/**: Contains main CLI entry points (parse.rs, search.rs)
- **src/config.rs**: Unified configuration management for .semtools_config.json (checks current dir, then home dir)
- **src/parse/**: Parse tool implementation with LlamaIndex API integration
  - `config.rs` - LlamaParse-specific configuration
  - `client.rs` - HTTP client for LlamaIndex API
  - `cache.rs` - File-based caching system using SHA-2 hashing
  - `backend.rs` - LlamaParseBackend trait implementation
- **cli/**: Node.js wrapper scripts for NPM distribution
- **scripts/install.js**: NPM post-install script for building Rust binaries

### Key Dependencies
- **Parse tool**: reqwest (HTTP), serde (JSON), tokio (async), sha2 (caching)
- **Search tool**: model2vec-rs (embeddings), simsimd (similarity computation)
- **Common**: anyhow (error handling), clap (CLI parsing)

### Configuration

- Tools look for `.semtools_config.json` in current directory first, then `~/.semtools_config.json`
- Environment variables: `LLAMA_CLOUD_API_KEY`, `OPENAI_API_KEY`
- Default embedding model: minishlab/potion-multilingual-128M (model2vec)
- Concurrent parsing with configurable request limits and retry logic

## Testing and Quality

### Running Tests
```bash
# Unit tests
cargo test

# Integration tests are in tests/ directory
cargo test --test parse_integration
cargo test --test search_integration
```

### Code Quality
Always run before submitting changes:
```bash
cargo clippy  # Linting
cargo fmt     # Formatting
cargo test    # All tests
```

## Development Patterns

### Error Handling
Uses `anyhow::Result` throughout for consistent error handling with context.

### CLI Design
Follows Unix philosophy:
- Read from stdin, write to stdout 
- Pipeline-friendly (parse output can feed search input)
- Use `println!` for output, `eprintln!` for errors
- Support --help with examples

### Feature Flags
Uses Cargo features for optional functionality:
- `default = ["parse", "search"]`
- `parse = [reqwest, serde, tokio, ...]` 
- `search = [model2vec-rs, simsimd]`