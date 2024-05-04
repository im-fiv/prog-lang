use proc_macro2 as pm2;
use quote::quote;

/// Expands to the type of an enum variant's fields.
pub(crate) fn expand_fields_type(variant: &syn::Variant) -> syn::Result<pm2::TokenStream> {
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
pub(crate) fn expand_destructure_pattern(fields_len: usize) -> (pm2::TokenStream, pm2::TokenStream) {
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
			value_names.push(syn::Ident::new(
				&format!("value{}", index),
				pm2::Span::call_site()
			));
		}

		let result = quote! { (#(#value_names),*) };
		(result.clone(), result)
	}
}