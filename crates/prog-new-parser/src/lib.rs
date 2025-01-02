mod stream;
pub mod token;
pub mod ast;

use anyhow::Result;
pub use stream::ParseStream;

pub trait Parse<'inp>: Sized {
	fn parse(input: &'inp ParseStream<'inp>) -> Result<Self>;
}
