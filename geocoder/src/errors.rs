#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    CSVError(#[from] csv::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
