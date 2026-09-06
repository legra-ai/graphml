//! Blocking reader over any `std::io::BufRead`.

use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::GraphmlError;
use crate::read::builder::Builder;
use crate::read::record::GraphmlRecord;

/// Streams `<node>` and `<edge>` records out of a GraphML document as
/// it is read. Memory is bounded by the `<key>` table plus one element.
pub struct GraphmlReader<R: BufRead> {
    reader: Reader<R>,
    builder: Builder,
    buf: Vec<u8>,
    done: bool,
}

impl<R: BufRead> GraphmlReader<R> {
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
    pub fn next_record(&mut self) -> Result<Option<GraphmlRecord>, GraphmlError> {
        while !self.done {
            self.buf.clear();
            let position = self.reader.buffer_position();
            let event = self
                .reader
                .read_event_into(&mut self.buf)
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

impl<R: BufRead> Iterator for GraphmlReader<R> {
    type Item = Result<GraphmlRecord, GraphmlError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_record().transpose()
    }
}
