use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum TaskError {
    #[error("Image task failed: {0}")]
    SingleError(String),

    #[error("Multiple operations failed: {0:?}")]
    BatchError(Vec<String>),

    #[error("No such task")]
    NoSuchTask,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Unknown shell")]
    UnknownShell,

    #[error("No images selected")]
    NoImages,
}
