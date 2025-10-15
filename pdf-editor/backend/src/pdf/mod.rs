pub mod content;
pub mod extract;
pub mod fonts;
pub mod loader;
pub mod patch;
pub mod write;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("lopdf error: {0}")]
    Lopdf(#[from] lopdf::Error),
    #[error("unsupported feature: {0}")]
    Unsupported(String),
    #[error("invalid document: {0}")]
    Invalid(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, PdfError>;
