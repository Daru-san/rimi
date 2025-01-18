use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TaskError {
    #[error("Image task failed: {0}")]
    SingleError(String),
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Unknown shell")]
    UnknownShell,

    #[error("No images selected")]
    NoImages,
}
