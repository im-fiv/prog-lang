use syn::{Result, Variant, Generics};

use proc_macro2::{TokenStream, Ident, Span};
use quote::quote;

/// Expands to the type of an enum variant's fields.
fn expand_fields_type(variant: &Variant) -> Result<TokenStream> {
	let unnamed_fields = match variant.fields {
		syn::Fields::Unnamed(ref unnamed_fields) => unnamed_fields,
		syn::Fields::Unit => return Ok(quote! { () }),

		_ => return Err(syn::Error::new_spanned(
			&variant.fields,
			"Derive of this macro is only allowed for enums with variants containing unnamed or unit fields"
		))
	};

	Ok(match variant.fields.len() {
		0 => quote! { () },

		1 => {
			let temp_type = &unnamed_fields.unnamed[0].ty;
			quote!(#temp_type)
		},

		_ => {
			let mut field_types = vec![];

			for field in &unnamed_fields.unnamed {
				field_types.push(&field.ty);
			}

			quote! {
				( #(#field_types),* )
			}
		}
	})
}

/// Expands to a token stream to destructure a variant with unnamed fields
fn expand_destructure_pattern(fields_len: usize) -> (TokenStream, TokenStream) {
	if fields_len == 0 {
		(
			quote! { },
			quote! { () }
		)
	} else if fields_len == 1 {
		(
			quote! { (value) },
			quote! { value }
		)
	} else {
		let mut value_names = vec![];

		for index in 0..fields_len {
			value_names.push(Ident::new(
				&format!("value{}", index),
				Span::call_site()
			));
		}

		let result = quote! { (#(#value_names),*) };
		(result.clone(), result)
	}
}

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