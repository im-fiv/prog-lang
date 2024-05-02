use syn::{Result, Variant, Generics};

use proc_macro2::{TokenStream, Ident};
use quote::quote;

use super::utils::{expand_fields_type, expand_destructure_pattern};

/// Expands to impl of `TryInto` for variant's unnamed fields type
pub(crate) fn expand_variant(
	variant: Variant,
	enum_name: &Ident,
	generics: &Generics
) -> Result<TokenStream> {
	let variant_name = variant.ident.clone();

	let fields_type = expand_fields_type(&variant)?;
	let (destructure_pattern, value_names) = expand_destructure_pattern(variant.fields.len());

	// Splitting generics data
	let (
		impl_generics,
		type_generics,
		where_clause
	) = generics.split_for_impl();

	Ok(quote! {
		impl #impl_generics ::std::convert::TryInto<#fields_type> for #enum_name #type_generics #where_clause {
			type Error = String;

			fn try_into(self) -> ::std::result::Result<#fields_type, Self::Error> {
				if let Self::#variant_name #destructure_pattern = self {
					return Ok(#value_names);
				}

				::std::result::Result::Err(::std::format!(
					"Variant `{}` of enum `{}` cannot be converted into `{}`",
					::std::stringify!(#variant_name),
					::std::stringify!(#enum_name),
					::std::stringify!(#fields_type)
				))
			}
		}
	})
}