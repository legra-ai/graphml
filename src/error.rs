//! GraphML parser error types.

/// Errors from parsing GraphML documents.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphmlError {
    /// The XML itself is malformed or could not be read.
    #[error("GraphML: XML error at byte {position}: {detail}")]
    Xml {
        /// Byte offset in the input where the reader stopped.
        position: u64,
        /// The XML parser's failure detail.
        detail: String,
    },

    /// The input ended while a `<node>`, `<edge>`, or `<data>` was
    /// still open.
    #[error("GraphML: unexpected end of document at byte {position}")]
    UnexpectedEof {
        /// Byte offset where the input ended.
        position: u64,
    },

    /// A required attribute is missing from an element.
    #[error("GraphML: <{element}> at byte {position} lacks the '{attribute}' attribute")]
    MissingAttribute {
        /// The element's local name.
        element: &'static str,
        /// The missing attribute.
        attribute: &'static str,
        /// Byte offset of the element.
        position: u64,
    },

    /// A `<data>` element references a key that was never declared.
    #[error("GraphML: <data key=\"{key}\"> at byte {position} references an undeclared <key>")]
    UndeclaredKey {
        /// The undeclared key id.
        key: String,
        /// Byte offset of the `<data>` element.
        position: u64,
    },

    /// A `<data>` element appears outside a `<node>` or `<edge>`.
    #[error("GraphML: <data> at byte {position} is not inside a <node> or <edge>")]
    DataOutsideElement {
        /// Byte offset of the `<data>` element.
        position: u64,
    },

    /// An edge carries no label: neither a `label` attribute nor a
    /// `<data>` for a key named `label`/`labels`.
    #[error("GraphML: edge {source_id} -> {target_id} has no label")]
    EdgeWithoutLabel {
        /// The edge's source node id.
        source_id: String,
        /// The edge's target node id.
        target_id: String,
    },
}

impl GraphmlError {
    pub(crate) fn xml(position: u64, error: &quick_xml::Error) -> Self {
        Self::Xml {
            position,
            detail: error.to_string(),
        }
    }
}
