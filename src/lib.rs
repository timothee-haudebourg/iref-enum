//! This is a companion crate for `iref` providing a derive macro to declare
//! enum types that converts into/from IRIs.
//!
//! Storage and comparison of IRIs can be costly. One may prefer the use of an enum
//! type representing known IRIs with cheap conversion functions between the two.
//! This crate provides a way to declare such enums in an simple way through the
//! use of a `IriEnum` derive macro.
//! This macro will implement `TryFrom<&Iri>` and `AsRef<Iri>` for you.
//!
//! ## Basic usage
//!
//! Use `#[derive(IriEnum)]` attribute to generate the implementation of
//! `TryFrom<&Iri>` and `AsRef<Iri>` for the enum type.
//! The IRI of each variant is defined with the `iri` attribute:
//! ```rust
//! use iref_enum::IriEnum;
//!
//! #[derive(IriEnum, PartialEq, Debug)]
//! pub enum Vocab {
//!   #[iri("https://schema.org/name")] Name,
//!   #[iri("https://schema.org/knows")] Knows
//! }
//!
//! let term: Vocab = static_iref::iri!("https://schema.org/name").try_into().unwrap();
//! assert_eq!(term, Vocab::Name)
//! ```
//!
//! Each variant must have at most one parameter.
//! If it has a parameter, its type must implement `TryFrom<&Iri>` and
//! `AsRef<Iri>`.
//!
//! ## Compact IRIs
//!
//! The derive macro also support compact IRIs using the special `iri_prefix` attribute.
//! First declare a prefix associated to a given `IRI`.
//! Then any `iri` attribute of the form `prefix:suffix` we be expanded into the concatenation of the prefix IRI and `suffix`.
//!
//! ```rust
//! # use iref_enum::IriEnum;
//! #[derive(IriEnum)]
//! #[iri_prefix("schema" = "https://schema.org/")]
//! pub enum Vocab {
//!   #[iri("schema:name")] Name,
//!   #[iri("schema:knows")] Knows
//! }
//! ```
use iref::IriBuf;
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use std::collections::HashMap;

macro_rules! error {
	( $( $x:expr ),* ) => {
		{
			let msg = format!($($x),*);
			let tokens: TokenStream = format!("compile_error!(\"{}\");", msg).parse().unwrap();
			tokens
		}
	};
}

fn filter_attribute(
	attr: syn::Attribute,
	name: &str,
) -> Result<Option<proc_macro2::TokenStream>, TokenStream> {
	if let Some(attr_id) = attr.path.get_ident() {
		if attr_id == name {
			if let Some(TokenTree::Group(group)) = attr.tokens.into_iter().next() {
				Ok(Some(group.stream()))
			} else {
				Err(error!("malformed `{}` attribute", name))
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
					if let Ok(iri) = IriBuf::new(concat) {
						return Ok(iri);
					} else {
						return Err(());
					}
				}
			}
		}
	}

	if let Ok(iri) = IriBuf::new(value.to_owned()) {
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
						if tokens.next().is_some() {
							if let Some(token) = tokens.next() {
								if let Ok(iri) = string_literal_token(token) {
									match IriBuf::new(iri) {
										Ok(iri) => {
											prefixes.insert(prefix, iri);
										}
										Err(e) => {
											return error!(
												"invalid IRI `{}` for prefix `{}`",
												e.0, prefix
											);
										}
									}
								} else {
									return error!("expected a string literal");
								}
							} else {
								return error!("expected a string literal");
							}
						} else {
							return error!("expected `=` literal");
						}
					} else {
						return error!("expected a string literal");
					}
				} else {
					return error!("expected a string literal");
				}
			}
			Ok(None) => (),
			Err(tokens) => return tokens,
		}
	}

	match ast.data {
		syn::Data::Enum(e) => {
			let type_id = ast.ident;
			let mut try_from = proc_macro2::TokenStream::new();
			let mut try_from_default = quote! { Err(()) };
			let mut into = proc_macro2::TokenStream::new();

			for variant in e.variants {
				let variant_ident = variant.ident;
				let mut variant_iri: Option<IriBuf> = None;

				for attr in variant.attrs {
					match filter_attribute(attr, "iri") {
						Ok(Some(tokens)) => match string_literal(tokens) {
							Ok(str) => {
								if let Ok(iri) = expand_iri(str.as_str(), &prefixes) {
									variant_iri = Some(iri)
								} else {
									return error!(
										"invalid IRI `{}` for variant `{}`",
										str, variant_ident
									);
								}
							}
							Err(_) => return error!("malformed `iri` attribute"),
						},
						Ok(None) => (),
						Err(tokens) => return tokens,
					}
				}

				match variant.fields {
					syn::Fields::Unit => {
						if let Some(iri) = variant_iri {
							let iri = iri.as_str();

							try_from.extend(quote! {
								_ if iri == static_iref::iri!(#iri) => Ok(#type_id::#variant_ident),
							});

							into.extend(quote! {
								#type_id::#variant_ident => static_iref::iri!(#iri),
							});
						} else {
							return error!("missing IRI for enum variant `{}`", variant_ident);
						}
					}
					syn::Fields::Named(_) => {
						return error!("variants with named fields are unsupported")
					}
					syn::Fields::Unnamed(fields) => {
						if fields.unnamed.len() == 1 {
							let field = fields.unnamed.into_iter().next().unwrap();
							let ty = field.ty;

							try_from_default = quote! {
								match #ty::try_from(iri) {
									Ok(value) => Ok(#type_id::#variant_ident(value)),
									Err(_) => {
										#try_from_default
									}
								}
							};

							into.extend(quote! {
								#type_id::#variant_ident(v) => v.into(),
							});
						} else {
							return error!(
								"variants with named more than one field are unsupported"
							);
						}
					}
				}
			}

			let output = quote! {
				impl<'a> ::std::convert::TryFrom<&'a ::iref::Iri> for #type_id {
					type Error = ();

					#[inline]
					fn try_from(iri: &'a ::iref::Iri) -> ::std::result::Result<#type_id, ()> {
						match iri {
							#try_from
							_ => #try_from_default
						}
					}
				}

				impl<'a, 'i> From<&'a #type_id> for &'i ::iref::Iri {
					#[inline]
					fn from(vocab: &'a #type_id) -> &'i ::iref::Iri {
						match vocab {
							#into
						}
					}
				}

				impl<'i> From<#type_id> for &'i ::iref::Iri {
					#[inline]
					fn from(vocab: #type_id) -> &'i ::iref::Iri {
						<&::iref::Iri as From<&#type_id>>::from(&vocab)
					}
				}

				impl<'a, 'i> From<&'a #type_id> for &'i ::iref::IriRef {
					#[inline]
					fn from(vocab: &'a #type_id) -> &'i ::iref::IriRef {
						<&::iref::Iri as From<&#type_id>>::from(vocab).as_iri_ref()
					}
				}

				impl<'i> From<#type_id> for &'i ::iref::IriRef {
					#[inline]
					fn from(vocab: #type_id) -> &'i ::iref::IriRef {
						<&::iref::Iri as From<#type_id>>::from(vocab).as_iri_ref()
					}
				}

				impl AsRef<iref::Iri> for #type_id {
					#[inline]
					fn as_ref(&self) -> &::iref::Iri {
						<&::iref::Iri as From<&#type_id>>::from(self)
					}
				}

				impl AsRef<iref::IriRef> for #type_id {
					#[inline]
					fn as_ref(&self) -> &::iref::IriRef {
						<&::iref::IriRef as From<&#type_id>>::from(self)
					}
				}
			};

			output.into()
		}
		_ => {
			error!("only enums are handled by IriEnum")
		}
	}
}

fn string_literal(tokens: proc_macro2::TokenStream) -> Result<String, &'static str> {
	if let Some(token) = tokens.into_iter().next() {
		string_literal_token(token)
	} else {
		Err("expected one string parameter")
	}
}

fn string_literal_token(token: proc_macro2::TokenTree) -> Result<String, &'static str> {
	if let TokenTree::Literal(lit) = token {
		let str = lit.to_string();

		if str.len() >= 2 {
			let mut buffer = String::with_capacity(str.len() - 2);
			for (i, c) in str.chars().enumerate() {
				if i == 0 || i == str.len() - 1 {
					if c != '"' {
						return Err("expected string literal");
					}
				} else {
					buffer.push(c)
				}
			}

			Ok(buffer)
		} else {
			Err("expected string literal")
		}
	} else {
		Err("expected string literal")
	}
}
