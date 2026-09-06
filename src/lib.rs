#![doc = include_str!("../README.md")]

mod error;
mod key;
#[cfg(feature = "property-graph-model")]
mod pg;
mod read;
mod types;

pub use error::GraphmlError;
pub use key::{KeyDomain, KeyType};
#[cfg(feature = "tokio")]
pub use read::AsyncGraphmlReader;
pub use read::{GraphmlReader, GraphmlRecord, parse_graphml, parse_graphml_with};
pub use types::{Edge, Node, Property};
