//! Tokio reader over any `AsyncBufRead` (feature `tokio`).

use quick_xml::Reader;
use quick_xml::events::Event;
use tokio::io::AsyncBufRead;

use crate::error::GraphmlError;
use crate::read::builder::Builder;
use crate::read::record::GraphmlRecord;

/// Async twin of [`GraphmlReader`](crate::GraphmlReader): streams
/// records out of a GraphML document read from a Tokio source, with the
/// same bounded memory.
pub struct AsyncGraphmlReader<R: AsyncBufRead + Unpin> {
    reader: Reader<R>,
    builder: Builder,
    buf: Vec<u8>,
    done: bool,
}

impl<R: AsyncBufRead + Unpin> AsyncGraphmlReader<R> {
    /// Start reading `input`.
    pub fn new(input: R) -> Self {
        let reader = Reader::from_reader(input);
        Self {
            reader,
            builder: Builder::default(),
            buf: Vec::new(),
            done: false,
        }
    }

    /// The next completed record, or `None` at the end of the document.
    ///
    /// # Errors
    ///
    /// Returns [`GraphmlError`] for malformed XML or GraphML.
    pub async fn next_record(&mut self) -> Result<Option<GraphmlRecord>, GraphmlError> {
        while !self.done {
            self.buf.clear();
            let position = self.reader.buffer_position();
            let event = self
                .reader
                .read_event_into_async(&mut self.buf)
                .await
                .map_err(|e| GraphmlError::xml(position, &e))?;
            if matches!(event, Event::Eof) {
                self.done = true;
                self.builder.finish(position)?;
                return Ok(None);
            }
            if let Some(record) = self.builder.handle(&event, position)? {
                return Ok(Some(record));
            }
        }
        Ok(None)
    }
}
