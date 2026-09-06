//! Whole-input convenience entry points.

use crate::error::GraphmlError;
use crate::read::record::GraphmlRecord;
use crate::read::sync_reader::GraphmlReader;
use crate::types::{
    Edge,
    Node,
};

/// Parse a complete GraphML document, invoking `on_node` / `on_edge`
/// for each record as it completes.
///
/// # Errors
///
/// Returns the first [`GraphmlError`] encountered.
pub fn parse_graphml_with(
    input: &str,
    mut on_node: impl FnMut(Node),
    mut on_edge: impl FnMut(Edge),
) -> Result<(), GraphmlError> {
    for record in GraphmlReader::new(input.as_bytes()) {
        match record? {
            GraphmlRecord::Node(node) => on_node(node),
            GraphmlRecord::Edge(edge) => on_edge(edge),
        }
    }
    Ok(())
}

/// Parse a complete GraphML document into its nodes and edges.
///
/// # Errors
///
/// Returns the first [`GraphmlError`] encountered.
pub fn parse_graphml(input: &str) -> Result<(Vec<Node>, Vec<Edge>), GraphmlError> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    parse_graphml_with(input, |node| nodes.push(node), |edge| edges.push(edge))?;
    Ok((nodes, edges))
}
