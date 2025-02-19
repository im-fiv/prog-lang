mod error;
pub use error::ProgError;

pub type ProgResult<'s, T> = Result<T, ProgError<'s>>;
