use syn::{Type, Ident, Token};
use proc_macro2 as pm2;
use quote::quote;

#[derive(Debug)]
pub(crate) struct GetArgumentInput {
	pub(crate) list_name: Ident,
	pub(crate) arg_name: Ident,
	pub(crate) inner_type: Option<Type>,
	pub(crate) optional: bool
}

impl syn::parse::Parse for GetArgumentInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let list_name = input.parse::<Ident>()?;

		input.parse::<Token![=>]>()?;

		let arg_name = input.parse::<Ident>()?;

		input.parse::<Token![:]>()?;

		let inner_type = if input.peek(Token![...]) {
			input.parse::<Token![...]>()?;
			None
		} else {
			Some(input.parse::<Type>()?)
		};

		let optional = input.peek(Token![?]);
		if optional {
			input.parse::<Token![?]>()?;
		}

		Ok(Self { list_name, arg_name, inner_type, optional })
	}
}

pub(crate) fn expand_optional(input: GetArgumentInput) -> pm2::TokenStream {
	let arg_name = input.arg_name;
	let list_name = input.list_name;
	let inner_type = input.inner_type.unwrap();

	// Note: double curly braces are extremely important
	quote! {{
		let arg_name = ::std::stringify!(#arg_name);
		let parsed_arg = #list_name.get(arg_name);

		parsed_arg.map(|parsed_arg| if let ParsedArg::Regular(inner_arg) = parsed_arg.to_owned() {
			::std::convert::TryInto::<#inner_type>
				::try_into(inner_arg)
				.unwrap_or_else(
					|_| ::std::unreachable!("Argument `{}` cannot be converted to `{}`", arg_name, ::std::stringify!(#inner_type))
				)
		} else {
			::std::unreachable!("Argument `{}` is not a regular argument", arg_name)
		})
	}}
}

pub(crate) fn expand_required(input: GetArgumentInput) -> pm2::TokenStream {
	let arg_name = input.arg_name.clone();
	let body = expand_optional(input);

	quote! {
		#body
		.unwrap_or_else(|| ::std::unreachable!("Argument `{}` does not exist", ::std::stringify!(#arg_name)))
	}
}

pub(crate) fn expand_variadic(input: GetArgumentInput) -> pm2::TokenStream {
	let arg_name = input.arg_name;
	let list_name = input.list_name;
	
	// Note: double curly braces are extremely important
	quote! {{
		let arg_name = ::std::stringify!(#arg_name);
		let parsed_arg = #list_name.get(arg_name);

		parsed_arg.map(|parsed_arg| if let ParsedArg::Variadic(var_args) = parsed_arg.to_owned() {
			var_args
		} else {
			::std::unreachable!("Argument `{}` is not a variadic argument", arg_name)
		}).unwrap_or_else(|| ::std::unreachable!("Argument `{}` does not exist", ::std::stringify!(arg_name)))
	}}
}