// SemTools library - provides document parsing and semantic search functionality

pub mod config;
pub use config::{AskConfig, SemtoolsConfig};

#[cfg(feature = "parse")]
pub mod parse;

#[cfg(feature = "parse")]
pub use parse::{
    DoclingBackend, DoclingConfig, DoclingServeBackend, DoclingServeConfig, JobError,
    LlamaParseBackend, LlamaParseConfig,
};

#[cfg(feature = "workspace")]
pub mod workspace;

#[cfg(feature = "search")]
pub mod search;

#[cfg(feature = "ask")]
pub mod ask;
