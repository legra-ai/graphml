//! The element state machine: consumes `quick-xml` events, remembers
//! `<key>` declarations, and completes one record per `</node>` or
//! `</edge>`. Memory is bounded by the key table plus one open element.

use quick_xml::XmlVersion;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{
    BytesStart,
    Event,
};

use crate::error::GraphmlError;
use crate::key::{
    KeyDecl,
    Keys,
    split_labels,
};
use crate::read::record::GraphmlRecord;
use crate::types::{
    Edge,
    Node,
    Property,
};

/// The element currently being assembled.
#[derive(Debug)]
enum Open {
    Node {
        id: String,
        labels: Vec<String>,
        properties: Vec<Property>,
    },
    Edge {
        id: Option<String>,
        source: String,
        target: String,
        label: Option<String>,
        properties: Vec<Property>,
    },
}

impl Open {
    fn push_data(&mut self, decl: &KeyDecl, value: String) {
        if decl.is_label() {
            match self {
                Self::Node { labels, .. } => labels.extend(split_labels(&value)),
                Self::Edge { label, .. } => {
                    *label = split_labels(&value).into_iter().next();
                }
            }
            return;
        }
        let property = Property {
            name: decl.name.clone(),
            value,
            type_hint: decl.attr_type.map(|t| t.as_str().to_owned()),
        };
        match self {
            Self::Node { properties, .. } | Self::Edge { properties, .. } => {
                properties.push(property);
            }
        }
    }

    fn finish(self) -> Result<GraphmlRecord, GraphmlError> {
        Ok(match self {
            Self::Node {
                id,
                labels,
                properties,
            } => GraphmlRecord::Node(Node {
                id,
                labels,
                properties,
            }),
            Self::Edge {
                id,
                source,
                target,
                label,
                properties,
            } => {
                let Some(label) = label.filter(|label| !label.is_empty()) else {
                    return Err(GraphmlError::EdgeWithoutLabel {
                        source_id: source,
                        target_id: target,
                    });
                };
                GraphmlRecord::Edge(Edge {
                    id,
                    source,
                    target,
                    label,
                    properties,
                })
            }
        })
    }
}

/// Feeds on XML events; yields a record when an element completes.
#[derive(Debug, Default)]
pub(super) struct Builder {
    keys: Keys,
    open: Option<Open>,
    /// The key of the `<data>` element whose text is being collected.
    data_key: Option<String>,
    text: String,
    /// Depth of `<graph>` nesting inside the open element (hierarchical
    /// GraphML): data of a nested graph belongs to no record.
    nested_graph_depth: usize,
}

impl Builder {
    /// Handle one event read at byte `position`.
    pub(super) fn handle(
        &mut self,
        event: &Event<'_>,
        position: u64,
    ) -> Result<Option<GraphmlRecord>, GraphmlError> {
        match event {
            Event::Start(start) => self.start(start, position, false),
            Event::Empty(start) => self.start(start, position, true),
            Event::End(end) => self.end(end.local_name().as_ref(), position),
            Event::Text(text) if self.data_key.is_some() => {
                self.text.push_str(&text.xml_content(XmlVersion::default()));
                Ok(None)
            }
            Event::CData(cdata) if self.data_key.is_some() => {
                self.text
                    .push_str(&cdata.xml_content(XmlVersion::default()));
                Ok(None)
            }
            Event::GeneralRef(reference) if self.data_key.is_some() => {
                let resolved = reference
                    .resolve_char_ref()
                    .map_err(|e| GraphmlError::xml(position, &e))?;
                if let Some(ch) = resolved {
                    self.text.push(ch);
                    return Ok(None);
                }
                let name = reference.xml_content(XmlVersion::default());
                let entity = resolve_predefined_entity(&name).ok_or_else(|| GraphmlError::Xml {
                    position,
                    detail: format!("unknown entity reference &{name};"),
                })?;
                self.text.push_str(entity);
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Called at end of input: an element still open means the
    /// document was truncated.
    pub(super) fn finish(&self, position: u64) -> Result<(), GraphmlError> {
        if self.open.is_some() || self.data_key.is_some() {
            return Err(GraphmlError::UnexpectedEof { position });
        }
        Ok(())
    }

    fn start(
        &mut self,
        start: &BytesStart<'_>,
        position: u64,
        empty: bool,
    ) -> Result<Option<GraphmlRecord>, GraphmlError> {
        match start.local_name().as_ref() {
            "key" => {
                let id = required(start, "key", "id", position)?;
                let decl = KeyDecl::new(
                    &id,
                    attribute(start, "attr.name")?.as_deref(),
                    attribute(start, "for")?.as_deref(),
                    attribute(start, "attr.type")?.as_deref(),
                );
                self.keys.insert(id, decl);
                Ok(None)
            }
            "graph" if self.open.is_some() => {
                self.nested_graph_depth += 1;
                Ok(None)
            }
            "node" if self.open.is_none() => {
                let id = required(start, "node", "id", position)?;
                self.open = Some(Open::Node {
                    id,
                    labels: Vec::new(),
                    properties: Vec::new(),
                });
                if empty { self.close() } else { Ok(None) }
            }
            "edge" if self.open.is_none() => {
                self.open = Some(Open::Edge {
                    id: attribute(start, "id")?,
                    source: required(start, "edge", "source", position)?,
                    target: required(start, "edge", "target", position)?,
                    label: attribute(start, "label")?,
                    properties: Vec::new(),
                });
                if empty { self.close() } else { Ok(None) }
            }
            "data" if self.nested_graph_depth == 0 => {
                if self.open.is_none() {
                    return Err(GraphmlError::DataOutsideElement { position });
                }
                let key = required(start, "data", "key", position)?;
                if self.keys.get(&key).is_none() {
                    return Err(GraphmlError::UndeclaredKey { key, position });
                }
                self.text.clear();
                if empty {
                    self.data_key = Some(key);
                    self.end_data();
                } else {
                    self.data_key = Some(key);
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn end(&mut self, name: &str, _position: u64) -> Result<Option<GraphmlRecord>, GraphmlError> {
        match name {
            "data" if self.data_key.is_some() => {
                self.end_data();
                Ok(None)
            }
            "graph" if self.nested_graph_depth > 0 => {
                self.nested_graph_depth -= 1;
                Ok(None)
            }
            "node" | "edge" if self.open.is_some() && self.nested_graph_depth == 0 => self.close(),
            _ => Ok(None),
        }
    }

    fn end_data(&mut self) {
        let Some(key) = self.data_key.take() else {
            return;
        };
        let value = std::mem::take(&mut self.text);
        if let (Some(decl), Some(open)) = (self.keys.get(&key), self.open.as_mut()) {
            open.push_data(decl, value);
        }
    }

    fn close(&mut self) -> Result<Option<GraphmlRecord>, GraphmlError> {
        self.open.take().map(Open::finish).transpose()
    }
}

fn attribute(start: &BytesStart<'_>, name: &str) -> Result<Option<String>, GraphmlError> {
    start
        .try_get_attribute(name)
        .map_err(|e| GraphmlError::Xml {
            position: 0,
            detail: e.to_string(),
        })?
        .map(|attr| {
            attr.normalized_value(XmlVersion::default())
                .map(std::borrow::Cow::into_owned)
                .map_err(|e| GraphmlError::Xml {
                    position: 0,
                    detail: e.to_string(),
                })
        })
        .transpose()
}

fn required(
    start: &BytesStart<'_>,
    element: &'static str,
    name: &'static str,
    position: u64,
) -> Result<String, GraphmlError> {
    attribute(start, name)?.ok_or(GraphmlError::MissingAttribute {
        element,
        attribute: name,
        position,
    })
}
