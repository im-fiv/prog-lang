use proc_macro2 as pm2;
use quote::quote;
use syn::{Ident, Token, Type};

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

		Ok(Self {
			list_name,
			arg_name,
			inner_type,
			optional
		})
	}
}

pub(crate) fn expand_optional(input: GetArgumentInput) -> pm2::TokenStream {
	let arg_name = input.arg_name;
	let list_name = input.list_name;
	let inner_type = input.inner_type.unwrap();

	let unwrap_pattern = match inner_type {
		Type::Infer(_) => quote! { inner_arg },
		_ => {
			quote! {
				match inner_arg {
					crate::Value::#inner_type(val) => val,
					_ => ::std::panic!("Argument `{arg_name}` is not a regular argument")
				}
			}
		}
	};

	quote! {
		{
			let arg_name = ::std::stringify!(#arg_name);
			let parsed_arg = #list_name.remove(arg_name);

			parsed_arg.map(|parsed_arg| {
				if let crate::arg_parser::ParsedArg::Regular(inner_arg) = parsed_arg {
					#unwrap_pattern
				} else {
					::std::panic!("Argument `{}` is not a regular argument", arg_name)
				}
			})
		}
	}
}

pub(crate) fn expand_required(input: GetArgumentInput) -> pm2::TokenStream {
	let arg_name = input.arg_name.clone();
	let body = expand_optional(input);

	quote! {
		#body
		.unwrap_or_else(|| {
			let arg_name = ::std::stringify!(#arg_name);
			::std::panic!("Argument `{arg_name}` does not exist")
		})
	}
}

pub(crate) fn expand_variadic(input: GetArgumentInput) -> pm2::TokenStream {
	let arg_name = input.arg_name;
	let list_name = input.list_name;

	quote! {
		{
			let arg_name = ::std::stringify!(#arg_name);
			let parsed_arg = #list_name.remove(arg_name);

			parsed_arg.map(|parsed_arg| {
				match parsed_arg {
					crate::arg_parser::ParsedArg::Variadic(var_args) => var_args,
					_ => ::std::panic!("Argument `{arg_name}` is not a variadic argument")
				}
			}).unwrap_or_else(|| ::std::panic!("Argument `{arg_name}` does not exist"))
		}
	}
}
