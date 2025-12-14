use thiserror::Error;

pub type Result<T> = std::result::Result<T, MyError>;

#[derive(Debug, Error)]
pub enum MyError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("Illegal Argument {name}: got {value}, expected: {expected}")]
    IllegalArgument {
        name: String,
        value: String,
        expected: String,
    },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
