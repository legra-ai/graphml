//! Conversions into the format-neutral `property-graph-model` records.
//!
//! Enabled by the `property-graph-model` feature. Each parsed record
//! maps field-for-field so a streaming import pipeline can hand the
//! parser's output straight to any consumer of `PgNode` / `PgEdge`.

use property_graph_model::{PgEdge, PgNode, PgProperty};

use crate::types::{Edge, Node, Property};

impl From<Property> for PgProperty {
    fn from(property: Property) -> Self {
        Self {
            key: property.name,
            value: property.value,
            type_hint: property.type_hint,
        }
    }
}

impl From<Node> for PgNode {
    fn from(node: Node) -> Self {
        Self {
            source_id: node.id,
            labels: node.labels,
            properties: node.properties.into_iter().map(PgProperty::from).collect(),
        }
    }
}

impl From<Edge> for PgEdge {
    fn from(edge: Edge) -> Self {
        Self {
            source_id: edge.source,
            target_id: edge.target,
            rel_type: edge.label,
            properties: edge.properties.into_iter().map(PgProperty::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use property_graph_model::{PgEdge, PgNode};

    use crate::parse_graphml;

    #[test]
    fn parsed_records_convert_field_for_field() {
        let input = r#"<graphml><key id="age" for="node" attr.name="age" attr.type="int"/><key id="labels" for="node" attr.name="labels"/><graph>
          <node id="1"><data key="labels">Person</data><data key="age">30</data></node>
          <edge source="1" target="2" label="KNOWS"/>
        </graph></graphml>"#;
        let (nodes, edges) = parse_graphml(input).expect("parse");
        let node = PgNode::from(nodes.into_iter().next().expect("one node"));
        assert_eq!(node.source_id, "1");
        assert_eq!(node.labels, vec!["Person"]);
        assert_eq!(node.properties[0].key, "age");
        assert_eq!(node.properties[0].type_hint.as_deref(), Some("int"));

        let edge = PgEdge::from(edges.into_iter().next().expect("one edge"));
        assert_eq!(edge.source_id, "1");
        assert_eq!(edge.target_id, "2");
        assert_eq!(edge.rel_type, "KNOWS");
    }
}
