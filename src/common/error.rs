use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Net(String),

    #[error("Database error: {0}")]
    Persistence(String),

    #[error("World-error: {0}")]
    World(String),

    #[error("Script error: {0}")]
    Script(String),

    #[error("Unknown error: {0}")]
    Other(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}


pub type Result<T> = std::result::Result<T, Error>;
