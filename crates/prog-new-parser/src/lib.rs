mod stream;
pub mod token;

use anyhow::Result;
use stream::ParseStream;

pub trait Parse: Sized {
	fn parse(input: &mut ParseStream) -> Result<Self>;
}
