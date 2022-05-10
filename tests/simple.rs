#![feature(proc_macro_hygiene)]

use iref_enum::IriEnum;
use static_iref::iri;

#[test]
fn try_from() {
	#[derive(IriEnum, PartialEq, Debug)]
	#[iri_prefix("schema" = "https://schema.org/")]
	pub enum Vocab {
		#[iri("schema:name")]
		Name,
		#[iri("schema:knows")]
		Knows,
	}

	assert_eq!(
		Vocab::try_from(iri!("https://schema.org/name")),
		Ok(Vocab::Name)
	);
	assert_eq!(
		Vocab::try_from(iri!("https://schema.org/knows")),
		Ok(Vocab::Knows)
	);
	assert_eq!(Vocab::try_from(iri!("https://schema.org/other")), Err(()))
}

#[test]
fn try_from_with_parameter() {
	#[derive(IriEnum, PartialEq, Debug)]
	#[iri_prefix("schema" = "https://schema.org/")]
	pub enum Vocab {
		#[iri("schema:name")]
		Name,
		#[iri("schema:knows")]
		Knows,
		Other(OtherVocab),
	}

	#[derive(IriEnum, PartialEq, Debug)]
	#[iri_prefix("schema" = "https://schema.org/")]
	pub enum OtherVocab {
		#[iri("schema:Text")]
		Text,
	}

	assert_eq!(
		Vocab::try_from(iri!("https://schema.org/name")),
		Ok(Vocab::Name)
	);
	assert_eq!(
		Vocab::try_from(iri!("https://schema.org/knows")),
		Ok(Vocab::Knows)
	);
	assert_eq!(
		Vocab::try_from(iri!("https://schema.org/Text")),
		Ok(Vocab::Other(OtherVocab::Text))
	);
	assert_eq!(Vocab::try_from(iri!("https://schema.org/other")), Err(()))
}
