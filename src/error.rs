use thiserror::Error;

pub type Result<T> = std::result::Result<T, MyError>;

#[derive(Debug, Error)]
pub enum MyError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Walkdir(#[from] walkdir::Error),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeMessagepackEncode(#[from] rmp_serde::encode::Error),
    #[error(transparent)]
    SerdeMessagepackDecode(#[from] rmp_serde::decode::Error),

    #[error("Illegal Argument {name}: got {value}, expected: {expected}")]
    IllegalArgument {
        name: String,
        value: String,
        expected: String,
    },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
