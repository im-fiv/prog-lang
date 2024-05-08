use syn::{Ident, Token};

#[derive(Debug)]
pub(crate) struct GetThisInput {
	pub(crate) this_arg_name: Ident,
	pub(crate) variant: Ident
}

impl syn::parse::Parse for GetThisInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let this_arg_name = input.parse::<Ident>()?;

		input.parse::<Token![=>]>()?;

		let variant = input.parse::<Ident>()?;

		Ok(Self { this_arg_name, variant })
	}
}