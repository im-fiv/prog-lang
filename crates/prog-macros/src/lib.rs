extern crate proc_macro;

mod conversion_inner;
mod get_argument_inner;
mod utils;

use proc_macro as pm;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Implements `TryInto` for all variants into their unnamed fields type. **Only compatible with enums**
#[proc_macro_derive(VariantUnwrap)]
pub fn variant_unwrap(item: pm::TokenStream) -> pm::TokenStream {
	let item = parse_macro_input!(item as DeriveInput);
	let enum_name = item.ident;

	// Unwrapping enum data
	let enum_data = match utils::get_enum_data(item.data) {
		Ok(data) => data,
		Err(err) => return err.to_compile_error().into()
	};

	// Expanding variants
	let mut expanded_variants = vec![];

	for variant in enum_data.variants {
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
	quote! {
		#(#expanded_variants)*
	}.into()
}

#[proc_macro_derive(EnumKind)]
pub fn enum_kind(item: pm::TokenStream) -> pm::TokenStream {
	let item = parse_macro_input!(item as DeriveInput);

	let enum_name = item.ident;
	let enum_vis = item.vis;
	let enum_kind_name = quote::format_ident!("{enum_name}Kind");

	// Unwrapping enum data
	let enum_data = match utils::get_enum_data(item.data) {
		Ok(data) => data,
		Err(err) => return err.to_compile_error().into()
	};
	
	// Expanding variants into their names
    let kind_variants = enum_data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! { #variant_name }
    });

	// Expanding display impl
	let enum_kind_display_impl = {
		let match_arms = enum_data.variants.iter().map(|variant| {
			let variant_name = &variant.ident;

			quote! {
				Self::#variant_name => ::core::write!(f, ::core::stringify!(#variant_name))
			}
		});

		quote! {
			impl ::std::fmt::Display for #enum_kind_name {
				fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
					match self {
						#( #match_arms ),*
					}
				}
			}
		}
	};

	// Expanding kind enum
	let kind_enum = quote! {
		#[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq)]
		#enum_vis enum #enum_kind_name {
			#( #kind_variants ),*
		}

		#enum_kind_display_impl
	};

	// Expanding `.kind()` for deriving enum
	let get_kind_impl = {
		let mut match_arms = vec![];

		// Assuming that the variant names are identical in the deriving enum and the kind enum
		for variant in enum_data.variants {
			let variant_name = variant.ident;

			let suffix = match variant.fields {
				syn::Fields::Named(_) => quote!( {..} ),
				syn::Fields::Unnamed(_) => quote!( (..) ),
				syn::Fields::Unit => quote!()
			};

			let match_arm = quote! {
				Self::#variant_name #suffix => #enum_kind_name::#variant_name
			};

			match_arms.push(match_arm);
		}

		quote! {
			impl #enum_name {
				pub fn kind(&self) -> #enum_kind_name {
					match self {
						#( #match_arms ),*
					}
				}
			}
		}
	};

	let expanded = quote! {
		#kind_enum
		#get_kind_impl
	};

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