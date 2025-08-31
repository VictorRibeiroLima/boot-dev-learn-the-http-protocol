use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ResponseWriterError {
    WriterAlreadyFlushed,
    WriterAlreadyClosed,
    WritingError(std::io::Error),
}

impl Display for ResponseWriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ResponseWriterError::WriterAlreadyFlushed => write!(f, "Writer already flushed"),
            ResponseWriterError::WriterAlreadyClosed => write!(f, "Writer already closed"),
            ResponseWriterError::WritingError(e) => write!(f, "Writing tcp stream error: {}", e),
        }
    }
}

impl Error for ResponseWriterError {}

impl From<std::io::Error> for ResponseWriterError {
    fn from(value: std::io::Error) -> Self {
        ResponseWriterError::WritingError(value)
    }
}
