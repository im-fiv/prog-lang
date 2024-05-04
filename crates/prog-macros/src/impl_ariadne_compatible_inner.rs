use proc_macro2 as pm2;
use quote::quote;

use syn::punctuated::Punctuated;
use syn::{Token, Variant, Error, Ident, ItemEnum};

pub(crate) fn expand_match_arms(variants: &Punctuated<Variant, Token![,]>, function_name: &str, arguments: Vec<&str>) -> syn::Result<Vec<pm2::TokenStream>> {
	let mut arms = vec![];

	for variant in variants {
		let variant_name = &variant.ident;

		match variant.fields {
			syn::Fields::Unnamed(_) => (),
			ref f => return Err(Error::new_spanned(f, "Expected unnamed fields"))
		};
		
		let function_name = Ident::new(function_name, pm2::Span::call_site());

		let arguments = arguments
			.iter()
			.map(|arg| Ident::new(arg, pm2::Span::call_site()))
			.collect::<Vec<_>>();

		arms.push(quote! {
			Self::#variant_name(value) => value.#function_name(#( #arguments ),*)
		});
	}

	Ok(arms)
}

pub(crate) fn expand_impl(item: ItemEnum) -> syn::Result<pm2::TokenStream> {
	let enum_name = item.ident;

	let message_match_arms = expand_match_arms(&item.variants, "message", vec![])?;
	let labels_match_arms = expand_match_arms(&item.variants, "labels", vec!["file", "position"])?;

	let expanded = quote! {
		impl ::prog_utils::pretty_errors::AriadneCompatible for #enum_name {
			fn message(&self) -> ::std::string::String {
				match self {
					#( #message_match_arms ),*
				}
			}

			fn labels(self, file: &str, position: ::prog_utils::pretty_errors::Position) -> ::std::vec::Vec<
				::ariadne::Label<
					::prog_utils::pretty_errors::Span
				>
			> {
				match self {
					#( #labels_match_arms ),*
				}
			}
		}
	};

	Ok(expanded)
}