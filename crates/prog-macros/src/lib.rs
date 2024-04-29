extern crate proc_macro;

mod conversion_inner;
mod get_argument_inner;

use proc_macro as pm;
use proc_macro2 as pm2;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Unwraps the enum data of a given item data if item is an enum
fn get_enum_data(item_data: syn::Data) -> syn::Result<syn::DataEnum> {
	let enum_data = match item_data {
		syn::Data::Enum(enum_data) => enum_data,

		// If item is not an enum, throw an error
		_ => return Err(syn::Error::new(
			pm2::Span::call_site(),
			"Derive of this macro is only allowed for enums"
		))
	};

	Ok(enum_data)
}

/// Implements `TryInto` for all variants into their unnamed fields type. **Only compatible with enums**
#[proc_macro_derive(Conversion)]
pub fn conversion(item: pm::TokenStream) -> pm::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let enum_name = item.ident;

    // Unwrapping enum data
	let data = match get_enum_data(item.data) {
		Ok(data) => data,
		Err(err) => return err.to_compile_error().into()
	};

    // Expanding variants
	let mut expanded_variants = vec![];

    for variant in data.variants {
		let expanded = conversion_inner::expand_variant(
			variant,
			&enum_name,
			&item.generics
		);

		expanded_variants.push(match expanded {
			Ok(expanded) => expanded,
			Err(err) => return err.to_compile_error().into()
		});
	}

    // Concatenating and returning
	let expanded = quote! { #(#expanded_variants)* };
	expanded.into()
}

/// Used for safely unwrapping a `ParsedArg`.
#[proc_macro]
pub fn get_argument(input: pm::TokenStream) -> pm::TokenStream {
	let input = parse_macro_input!(input as get_argument_inner::GetArgumentInput);

	if input.inner_type.is_none() {
		return get_argument_inner::expand_variadic(input).into();
	}

	if input.optional {
		return get_argument_inner::expand_optional(input).into();
	}

	get_argument_inner::expand_required(input).into()
}