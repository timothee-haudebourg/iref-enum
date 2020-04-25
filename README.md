# IRI Enums

<table><tr>
	<td><a href="https://docs.rs/iref-enum">Documentation</a></td>
	<td><a href="https://crates.io/crates/iref-enum">Crate informations</a></td>
	<td><a href="https://github.com/timothee-haudebourg/iref-enum">Repository</a></td>
</tr></table>

This is a companion crate for `iref` providing a derive macro to declare
enum types that converts into/from IRIs.

Storage and comparison of IRIs can be costly. One may prefer the use of an enum
type representing known IRIs with cheap convertion functions between the two.
This crate provides a way to declare such enums in an simple way through the
use of a `IriEnum` derive macro.
This macro will implement `TryFrom<Iri>` and `Into<Iri>` for you.

## Basic usage

Use `#[derive(IriEnum)]` attribute to generate the implementation of
`TryFrom<Iri>` and `Into<Iri>` for the enum type.
The IRI of each variant is defined with the `iri` attribute:
```rust
#[macro_use]
extern crate iref_enum;
use std::convert::TryInto;

#[derive(IriEnum, PartialEq, Debug)]
pub enum Vocab {
	#[iri("http://xmlns.com/foaf/0.1/name")] Name,
	#[iri("http://xmlns.com/foaf/0.1/knows")] Knows
}

pub fn main() {
	let term: Vocab = static_iref::iri!("http://xmlns.com/foaf/0.1/name").try_into().unwrap();
	assert_eq!(term, Vocab::Name)
}
```

## Compact IRIs

The derive macro also support compact IRIs using the special `iri_prefix` attribute.
First declare a prefix associated to a given `IRI`.
Then any `iri` attribute of the form `prefix:suffix` we be expanded into the concatenation of the prefix IRI and `suffix`.

```rust
#[derive(IriEnum)]
#[iri_prefix("foaf" = "http://xmlns.com/foaf/0.1/")]
pub enum Vocab {
	#[iri("foaf:name")] Name,
	#[iri("foaf:knows")] Knows
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
