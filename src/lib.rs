//! # IRI Enums
//!
//! <table><tr>
//! 	<td><a href="https://docs.rs/iref-enum">Documentation</a></td>
//! 	<td><a href="https://crates.io/crates/iref-enum">Crate informations</a></td>
//! 	<td><a href="https://github.com/timothee-haudebourg/iref-enum">Repository</a></td>
//! </tr></table>
//!
//! This is a companion crate for `iref` providing a derive macro to declare
//! enum types that converts into/from IRIs.
//!
//! Storage and comparison of IRIs can be costly. One may prefer the use of an enum
//! type representing known IRIs with cheap convertion functions between the two.
//! This crate provides a way to declare such enums in an simple way through the
//! use of a `IriEnum` derive macro.
//! This macro will implement `TryFrom<Iri>` and `Into<Iri>` for you.
//!
//! ## Basic usage
//!
//! Use `#[derive(IriEnum)]` attribute to generate the implementation of
//! `TryFrom<Iri>` and `Into<Iri>` for the enum type.
//! The IRI of each variant is defined with the `iri` attribute:
//! ```rust
//! #[macro_use]
//! extern crate iref_enum;
//! use std::convert::TryInto;
//!
//! #[derive(IriEnum, PartialEq, Debug)]
//! pub enum Vocab {
//! 	#[iri("http://xmlns.com/foaf/0.1/name")] Name,
//! 	#[iri("http://xmlns.com/foaf/0.1/knows")] Knows
//! }
//!
//! pub fn main() {
//! 	let term: Vocab = static_iref::iri!("http://xmlns.com/foaf/0.1/name").try_into().unwrap();
//! 	assert_eq!(term, Vocab::Name)
//! }
//! ```
//!
//! ## Compact IRIs
//!
//! The derive macro also support compact IRIs using the special `iri_prefix` attribute.
//! First declare a prefix associated to a given `IRI`.
//! Then any `iri` attribute of the form `prefix:suffix` we be expanded into the concatenation of the prefix IRI and `suffix`.
//!
//! ```rust
//! #[derive(IriEnum)]
//! #[iri_prefix("foaf" = "http://xmlns.com/foaf/0.1/")]
//! pub enum Vocab {
//! 	#[iri("foaf:name")] Name,
//! 	#[iri("foaf:knows")] Knows
//! }
//! ```
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn;
use std::collections::HashMap;
use iref::IriBuf;

macro_rules! error {
	( $( $x:expr ),* ) => {
		{
			let msg = format!($($x),*);
			let tokens: TokenStream = format!("compile_error!(\"{}\");", msg).parse().unwrap();
			tokens
		}
	};
}

fn filter_attribute(attr: syn::Attribute, name: &str) -> Result<Option<proc_macro2::TokenStream>, TokenStream> {
	if let Some(attr_id) = attr.path.get_ident() {
		if attr_id == name {
			if let Some(TokenTree::Group(group)) = attr.tokens.into_iter().next() {
				Ok(Some(group.stream()))
			} else {
				return Err(error!("malformed `{}` attribute", name))
			}
		} else {
			Ok(None)
		}
	} else {
		Ok(None)
	}
}

fn expand_iri(value: &str, prefixes: &HashMap<String, IriBuf>) -> Result<IriBuf, ()> {
	if let Some(index) = value.find(':') {
		if index > 0 {
			let (prefix, suffix) = value.split_at(index);
			let suffix = &suffix[1..suffix.len()];

			if !suffix.starts_with("//") {
				if let Some(base_iri) = prefixes.get(prefix) {
					let concat = base_iri.as_str().to_string() + suffix;
					if let Ok(iri) = IriBuf::new(concat.as_str()) {
						return Ok(iri)
					} else {
						return Err(())
					}
				}
			}
		}
	}

	if let Ok(iri) = IriBuf::new(value) {
		Ok(iri)
	} else {
		Err(())
	}
}

#[proc_macro_derive(IriEnum, attributes(iri_prefix, iri))]
pub fn iri_enum_derive(input: TokenStream) -> TokenStream {
	let ast: syn::DeriveInput = syn::parse(input).unwrap();

	let mut prefixes = HashMap::new();
	for attr in ast.attrs {
		match filter_attribute(attr, "iri_prefix") {
			Ok(Some(tokens)) => {
				let mut tokens = tokens.into_iter();
				if let Some(token) = tokens.next() {
					if let Ok(prefix) = string_literal_token(token) {
						if let Some(_) = tokens.next() {
							if let Some(token) = tokens.next() {
								if let Ok(iri) = string_literal_token(token) {
									if let Ok(iri) = IriBuf::new(iri.as_str()) {
										prefixes.insert(prefix, iri);
									} else {
										return error!("invalid IRI `{}` for prefix `{}`", iri, prefix)
									}
								} else {
									return error!("expected a string literal")
								}
							} else {
								return error!("expected a string literal")
							}
						} else {
							return error!("expected `=` literal")
						}
					} else {
						return error!("expected a string literal")
					}
				} else {
					return error!("expected a string literal")
				}
			},
			Ok(None) => (),
			Err(tokens) => return tokens
		}
	}

	match ast.data {
		syn::Data::Enum(e) => {
			let type_id = ast.ident;
			let mut try_from = proc_macro2::TokenStream::new();
			let mut into = proc_macro2::TokenStream::new();

			for variant in e.variants {
				let variant_ident = variant.ident;
				let mut variant_iri: Option<IriBuf> = None;

				for attr in variant.attrs {
					match filter_attribute(attr, "iri") {
						Ok(Some(tokens)) => {
							match string_literal(tokens) {
								Ok(str) => {
									if let Ok(iri) = expand_iri(str.as_str(), &prefixes) {
										variant_iri = Some(iri)
									} else {
										return error!("invalid IRI `{}` for variant `{}`", str, variant_ident)
									}
								},
								Err(_) => {
									return error!("malformed `iri` attribute")
								}
							}
						},
						Ok(None) => (),
						Err(tokens) => return tokens
					}
				}

				if let Some(iri) = variant_iri {
					let iri = iri.as_str();

					try_from.extend(quote! {
						_ if iri == static_iref::iri!(#iri) => Ok(#type_id::#variant_ident),
					});

					into.extend(quote! {
						#type_id::#variant_ident => static_iref::iri!(#iri),
					});
				} else {
					return error!("missing IRI for enum variant `{}`", variant_ident)
				}
			}

			let output = quote! {
				impl ::std::convert::TryFrom<::iref::Iri<'_>> for #type_id {
					type Error = ();

					fn try_from(iri: ::iref::Iri) -> ::std::result::Result<#type_id, ()> {
						match iri {
							#try_from
							_ => Err(())
						}
					}
				}

				impl<'a> Into<::iref::Iri<'a>> for &'a #type_id {
					fn into(self) -> ::iref::Iri<'a> {
						match self {
							#into
						}
					}
				}
			};

			output.into()
		},
		_ => {
			error!("only enums are handled by IriEnum")
		}
	}
}

fn string_literal(tokens: proc_macro2::TokenStream) -> Result<String, &'static str> {
	if let Some(token) = tokens.into_iter().next() {
		string_literal_token(token)
	} else {
		return Err("expected one string parameter");
	}
}

fn string_literal_token(token: proc_macro2::TokenTree) -> Result<String, &'static str> {
	if let TokenTree::Literal(lit) = token {
		let str = lit.to_string();

		if str.len() >= 2 {
			let mut buffer = String::with_capacity(str.len()-2);
			for (i, c) in str.chars().enumerate() {
				if i == 0 || i == str.len()-1 {
					if c != '"' {
						return Err("expected string literal");
					}
				} else {
					buffer.push(c)
				}
			}

			Ok(buffer)
		} else {
			return Err("expected string literal");
		}
	} else {
		return Err("expected string literal");
	}
}
