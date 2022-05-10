use iref_enum::IriEnum;
use static_iref::iri;

#[derive(IriEnum, PartialEq, Debug)]
#[iri_prefix("schema" = "https://schema.org/")]
pub enum Vocab {
	#[iri("schema:name")]
	Name,
	#[iri("schema:knows")]
	Knows,
}

pub fn main() {
	let term: Vocab = iri!("https://schema.org/name").try_into().unwrap();
	assert_eq!(term, Vocab::Name)
}
