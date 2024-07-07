extern crate proc_macro;

mod conversion_inner;
mod get_argument_inner;
mod get_this_inner;
mod impl_ariadne_compatible_inner;
mod utils;

use proc_macro as pm;
use quote::quote;
use syn::parse_macro_input;

/// Implements [`TryInto`] for all variants into their unnamed fields type. **Only compatible with enums**
#[proc_macro_derive(VariantUnwrap)]
pub fn variant_unwrap(input: pm::TokenStream) -> pm::TokenStream {
	let item = parse_macro_input!(input as syn::ItemEnum);
	let enum_name = item.ident;

	// Expanding variants
	let mut expanded_variants = vec![];

	for variant in item.variants {
		let expanded = conversion_inner::expand_variant(variant, &enum_name, &item.generics);

		expanded_variants.push(match expanded {
			Ok(expanded) => expanded,
			Err(err) => return err.to_compile_error().into()
		});
	}

	// Concatenating and returning
	quote! {
		#( #expanded_variants )*
	}
	.into()
}

/// Expands to `enum <Enum>Kind` and implements `.kind()` for the deriving enum. **Only compatible with enums**
#[proc_macro_derive(EnumKind)]
pub fn enum_kind(input: pm::TokenStream) -> pm::TokenStream {
	let item = parse_macro_input!(input as syn::ItemEnum);

	let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

	let enum_name = item.ident;
	let enum_vis = item.vis;
	let enum_kind_name = quote::format_ident!("{enum_name}Kind");

	// Expanding variants into their names
	let kind_variants = item.variants.iter().map(|variant| {
		let variant_name = &variant.ident;
		quote! { #variant_name }
	});

	// Expanding display impl
	let enum_kind_display_impl = {
		let match_arms = item.variants.iter().map(|variant| {
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
		#[cfg_attr(feature = "serialize", derive(::serde::Serialize))]
		#enum_vis enum #enum_kind_name {
			#( #kind_variants ),*
		}

		#enum_kind_display_impl
	};

	// Expanding `.kind()` for deriving enum
	let get_kind_impl = {
		let mut match_arms = vec![];

		// Assuming that the variant names are identical in the deriving enum and the kind enum
		for variant in item.variants {
			let variant_name = variant.ident;

			let suffix = match variant.fields {
				syn::Fields::Named(_) => quote!({ .. }),
				syn::Fields::Unnamed(_) => quote!((..)),
				syn::Fields::Unit => quote!()
			};

			let match_arm = quote! {
				Self::#variant_name #suffix => #enum_kind_name::#variant_name
			};

			match_arms.push(match_arm);
		}

		quote! {
			impl #impl_generics #enum_name #type_generics #where_clause {
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

/// Expands to the implementation of [`prog_utils::pretty_errors::AriadneCompatible`]. **Only compatible with enums**
#[proc_macro_derive(ImplAriadneCompatible)]
pub fn impl_ariadne_compatible(input: pm::TokenStream) -> pm::TokenStream {
	let item = parse_macro_input!(input as syn::ItemEnum);

	match impl_ariadne_compatible_inner::expand_impl(item) {
		Ok(ts) => ts.into(),
		Err(e) => e.to_compile_error().into()
	}
}

/// Used for safe unwrapping of a [`prog_interpreter::arg_parser::ParsedArg`].
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

/// Used for safe unwrapping of a [`prog_interpreter::values::IntrinsicFunction`] `this` arg
#[proc_macro]
pub fn get_this(input: pm::TokenStream) -> pm::TokenStream {
	let input = parse_macro_input!(input as get_this_inner::GetThisInput);
	let get_this_inner::GetThisInput {
		this_arg_name,
		variant
	} = input;

	// Note: double curly braces are extremely important
	quote! {{
		let this = #this_arg_name.unwrap_or_else(|| ::std::unreachable!("`this` argument is `None`"));

		if let crate::values::RuntimeValue::#variant(inner_this) = this {
			inner_this
		} else {
			::std::unreachable!("`this` argument is not of variant `{}`", stringify!(#variant))
		}
	}}
	.into()
}
