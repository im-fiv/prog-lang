use proc_macro2 as pm2;
use pm2::Span;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Error, Ident, ItemEnum, Token, Variant};

fn expand_match_arms(
	variants: &Punctuated<Variant, Token![,]>,
	function_name: &str,
	arguments: Vec<&str>
) -> syn::Result<Vec<pm2::TokenStream>> {
	let mut arms = vec![];

	for variant in variants {
		let variant_name = &variant.ident;

		match variant.fields {
			syn::Fields::Unnamed(_) => (),
			ref f => return Err(Error::new_spanned(f, "Expected unnamed fields"))
		};

		let function_name = Ident::new(function_name, Span::call_site());

		let arguments = arguments
			.iter()
			.map(|arg| Ident::new(arg, Span::call_site()))
			.collect::<Vec<_>>();

		arms.push(quote! {
			Self::#variant_name(value) => value.#function_name(#( #arguments ),*)
		});
	}

	Ok(arms)
}

pub(crate) fn expand_impl(item: ItemEnum) -> syn::Result<pm2::TokenStream> {
	let trait_lifetime = syn::Lifetime::new(
		"'__lt_AriadneCompatible_1",
		Span::call_site()
	);
	
	let mut impl_generics = item.generics.clone();

	// Adding the `: AriadneCompatibleLifetime` bound to all other lifetimes
	for lt_param in impl_generics.lifetimes_mut() {
		lt_param.bounds.push(trait_lifetime.clone());
	}

	// Adding `AriadneCompatible`'s lifetime to the `impl` params
	impl_generics.params.insert(0, syn::GenericParam::Lifetime(
		syn::LifetimeParam {
			attrs: vec![],
			lifetime: trait_lifetime.clone(),
			colon_token: None,
			bounds: Punctuated::new()
		}
	));

	let enum_name = item.ident;
	let (impl_generics, _, _) = impl_generics.split_for_impl();
	let (_, type_generics, where_clause) = item.generics.split_for_impl();

	let message_match_arms = expand_match_arms(&item.variants, "message", vec![])?;
	let labels_match_arms = expand_match_arms(&item.variants, "labels", vec!["span"])?;

	let expanded = quote! {
		#[allow(non_snake_case)]
		impl #impl_generics ::prog_utils::pretty_errors::AriadneCompatible<#trait_lifetime> for #enum_name #type_generics #where_clause {
			fn message(&self) -> &'static str {
				match self {
					#( #message_match_arms ),*
				}
			}

			fn labels(&self, span: ::prog_utils::pretty_errors::Span<#trait_lifetime>) -> ::std::vec::Vec<
				::ariadne::Label<
					::prog_utils::pretty_errors::Span<#trait_lifetime>
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
