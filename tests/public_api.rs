//! Public-API integration test: the shipped crate parses a Neo4j-style
//! GraphML export through both the sync and the async readers.

use graphml::{
    GraphmlError,
    GraphmlReader,
    GraphmlRecord,
    parse_graphml,
};

const EXPORT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <key id="labels" for="node" attr.name="labels"/>
  <key id="name" for="node" attr.name="name"/>
  <key id="since" for="edge" attr.name="since" attr.type="int"/>
  <graph id="G" edgedefault="directed">
    <node id="n0"><data key="labels">:Person</data><data key="name">Alice &amp; Bob</data></node>
    <node id="n1"><data key="labels">:Person:Employee</data><data key="name">Carol</data></node>
    <edge id="e0" source="n0" target="n1" label="KNOWS"><data key="since">2020</data></edge>
  </graph>
</graphml>
"#;

#[test]
fn whole_document_parse_yields_typed_nodes_and_edges() {
    let (nodes, edges) = parse_graphml(EXPORT).expect("valid GraphML");
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].id, "n0");
    assert_eq!(nodes[0].labels, ["Person"]);
    assert_eq!(nodes[0].properties[0].value, "Alice & Bob");
    assert_eq!(nodes[1].labels, ["Person", "Employee"]);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].label, "KNOWS");
    assert_eq!(edges[0].properties[0].type_hint.as_deref(), Some("int"));
}

#[test]
fn streaming_reader_yields_records_in_document_order() {
    let kinds: Vec<&str> = GraphmlReader::new(EXPORT.as_bytes())
        .map(|record| match record.expect("valid GraphML") {
            GraphmlRecord::Node(_) => "node",
            GraphmlRecord::Edge(_) => "edge",
        })
        .collect();
    assert_eq!(kinds, ["node", "node", "edge"]);
}

#[test]
fn undeclared_keys_are_rejected_not_dropped() {
    let error = parse_graphml(
        r#"<graphml><graph><node id="a"><data key="ghost">1</data></node></graph></graphml>"#,
    )
    .expect_err("undeclared key");
    assert!(matches!(error, GraphmlError::UndeclaredKey { ref key, .. } if key == "ghost"));
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn async_reader_matches_the_sync_reader() {
    let mut reader = graphml::AsyncGraphmlReader::new(EXPORT.as_bytes());
    let mut count = 0;
    while reader.next_record().await.expect("valid GraphML").is_some() {
        count += 1;
    }
    assert_eq!(count, 3);
}
