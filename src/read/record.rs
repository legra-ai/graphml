//! The parsed record of one completed element.

use crate::types::{Edge, Node};

/// One completed `<node>` or `<edge>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphmlRecord {
    /// A completed `<node>`.
    Node(Node),
    /// A completed `<edge>`.
    Edge(Edge),
}
