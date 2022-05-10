#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate iref_enum;
#[macro_use]
extern crate static_iref;

use std::convert::TryInto;

#[derive(IriEnum, PartialEq, Debug)]
#[iri_prefix("foaf" = "http://xmlns.com/foaf/0.1/")]
pub enum Vocab {
	#[iri("foaf:name")]
	Name,
	#[iri("foaf:knows")]
	Knows,
}

pub fn main() {
	let term: Vocab = iri!("http://xmlns.com/foaf/0.1/name").try_into().unwrap();
	assert_eq!(term, Vocab::Name)
}
