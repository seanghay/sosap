//! Pure-Rust port of Phonetisaurus G2P inference.
//!
//! Mirrors the upstream `PhonetisaurusScript` C++ API surface.

mod symbols;
mod tokenize;
mod fsa;
mod path_filter;
mod nbest;
mod decode;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod wasm;

#[cfg(feature = "python")]
mod python;

pub use decode::{Model, PathData, PhoneticizeOptions};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "std-fs")]
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to read FST: {0}")]
    FstRead(String),
    #[error("FST is missing {0} symbol table")]
    MissingSymbolTable(&'static str),
}
