#[derive(Debug, thiserror::Error)]
pub enum NpsError {
    #[error("Internal Server Error")]
    ServerInternal,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
