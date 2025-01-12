mod error;
pub use error::ProgError;

pub type ProgResult<'kind, T> = Result<T, ProgError<'kind>>;
