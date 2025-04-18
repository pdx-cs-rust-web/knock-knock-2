extern crate serde_json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KnockKnockError {
    #[error("could not find joke file: {0}")]
    JokesNotFound(#[from] std::io::Error),
    #[error("could not read joke file: {0}")]
    JokeMisformat(#[from] serde_json::Error),
}
