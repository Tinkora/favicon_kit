use thiserror::Error;

/// Stable error type for favicon generation.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    /// Source image is below the minimum allowed dimension.
    #[error("Image too small: {0}×{1}. Minimum dimension is {2}px")]
    ImageTooSmall(u32, u32, u32),

    /// Source image exceeds the maximum allowed dimension.
    #[error("Image too large: {0}×{1}. Maximum dimension is {2}px")]
    ImageTooLarge(u32, u32, u32),

    /// Image resize operation failed.
    #[error("Resize error: {0}")]
    ResizeError(String),

    /// ICO encoding failed.
    #[error("ICO encode error: {0}")]
    IcoError(String),

    /// ZIP creation failed.
    #[error("ZIP export error: {0}")]
    ZipError(String),
}

impl CoreError {
    /// Returns a stable machine-readable error code for JS consumers.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::ImageTooSmall(..) => "IMAGE_TOO_SMALL",
            Self::ImageTooLarge(..) => "IMAGE_TOO_LARGE",
            Self::ResizeError(_) => "RESIZE_ERROR",
            Self::IcoError(_) => "ICO_ERROR",
            Self::ZipError(_) => "ZIP_ERROR",
        }
    }
}
