//! Event-driven reading: a shared state machine fed by a sync or an
//! async `quick-xml` reader.

#[cfg(feature = "tokio")]
mod async_reader;
mod builder;
mod entry;
mod record;
mod sync_reader;

#[cfg(test)]
mod tests;

#[cfg(feature = "tokio")]
pub use async_reader::AsyncGraphmlReader;
pub use entry::{parse_graphml, parse_graphml_with};
pub use record::GraphmlRecord;
pub use sync_reader::GraphmlReader;
