mod error;
pub use error::ProgError;

pub type ProgResult<T> = Result<T, ProgError>;