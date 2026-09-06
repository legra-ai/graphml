use crate::error::GraphmlError;
use crate::read::{GraphmlReader, GraphmlRecord, parse_graphml};

const NEO4J_EXPORT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <key id="labels" for="node" attr.name="labels"/>
  <key id="name" for="node" attr.name="name"/>
  <key id="age" for="node" attr.name="age" attr.type="int"/>
  <key id="label" for="edge" attr.name="label"/>
  <key id="since" for="edge" attr.name="since" attr.type="long"/>
  <graph id="G" edgedefault="directed">
    <node id="n0" labels=":Person"><data key="labels">:Person:Employee</data><data key="name">Alice &amp; co</data><data key="age">30</data></node>
    <node id="n1"><data key="labels">:Person</data><data key="name"><![CDATA[Bob <b>]]></data></node>
    <edge id="e0" source="n0" target="n1" label="KNOWS"><data key="label">KNOWS</data><data key="since">2020</data></edge>
  </graph>
</graphml>
"#;

#[test]
fn neo4j_style_export_parses_into_labelled_nodes_and_typed_properties() {
    let (nodes, edges) = parse_graphml(NEO4J_EXPORT).expect("parse");
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].id, "n0");
    assert_eq!(nodes[0].labels, ["Person", "Employee"]);
    assert_eq!(nodes[0].properties.len(), 2);
    assert_eq!(nodes[0].properties[0].name, "name");
    assert_eq!(nodes[0].properties[0].value, "Alice & co");
    assert_eq!(nodes[0].properties[0].type_hint, None);
    assert_eq!(nodes[0].properties[1].value, "30");
    assert_eq!(nodes[0].properties[1].type_hint.as_deref(), Some("int"));
    assert_eq!(nodes[1].properties[0].value, "Bob <b>");
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].id.as_deref(), Some("e0"));
    assert_eq!(edges[0].source, "n0");
    assert_eq!(edges[0].target, "n1");
    assert_eq!(edges[0].label, "KNOWS");
    assert_eq!(edges[0].properties.len(), 1);
    assert_eq!(edges[0].properties[0].type_hint.as_deref(), Some("long"));
}

#[test]
fn records_stream_one_at_a_time() {
    let mut reader = GraphmlReader::new(NEO4J_EXPORT.as_bytes());
    assert!(matches!(
        reader.next_record(),
        Ok(Some(GraphmlRecord::Node(_)))
    ));
    assert!(matches!(
        reader.next_record(),
        Ok(Some(GraphmlRecord::Node(_)))
    ));
    assert!(matches!(
        reader.next_record(),
        Ok(Some(GraphmlRecord::Edge(_)))
    ));
    assert!(matches!(reader.next_record(), Ok(None)));
    assert!(matches!(reader.next_record(), Ok(None)), "stays exhausted");
}

#[test]
fn edge_label_attribute_alone_suffices_and_empty_elements_work() {
    let input = r#"<graphml><graph><node id="a"/><node id="b"/><edge source="a" target="b" label="REL"/></graph></graphml>"#;
    let (nodes, edges) = parse_graphml(input).expect("parse");
    assert_eq!(nodes.len(), 2);
    assert_eq!(edges[0].label, "REL");
    assert_eq!(edges[0].id, None);
}

#[test]
fn nested_graphs_do_not_leak_data_into_the_parent_node() {
    let input = r#"<graphml><key id="k" for="node" attr.name="k"/><graph>
      <node id="outer"><data key="k">outer</data><graph id="sub"><node id="inner"><data key="k">inner</data></node></graph></node>
    </graph></graphml>"#;
    let (nodes, _) = parse_graphml(input).expect("parse");
    assert_eq!(
        nodes.len(),
        1,
        "inner nodes of a nested graph are not emitted: {nodes:?}"
    );
    assert_eq!(nodes[0].id, "outer");
    assert_eq!(nodes[0].properties.len(), 1);
    assert_eq!(nodes[0].properties[0].value, "outer");
}

#[test]
fn structural_errors_are_typed() {
    let missing = parse_graphml(r"<graphml><graph><node/></graph></graphml>").unwrap_err();
    assert!(
        matches!(
            missing,
            GraphmlError::MissingAttribute {
                element: "node",
                attribute: "id",
                ..
            }
        ),
        "{missing}"
    );
    let undeclared = parse_graphml(
        r#"<graphml><graph><node id="a"><data key="zz">1</data></node></graph></graphml>"#,
    )
    .unwrap_err();
    assert!(
        matches!(undeclared, GraphmlError::UndeclaredKey { ref key, .. } if key == "zz"),
        "{undeclared}"
    );
    let unlabelled =
        parse_graphml(r#"<graphml><graph><edge source="a" target="b"/></graph></graphml>"#)
            .unwrap_err();
    assert_eq!(
        unlabelled,
        GraphmlError::EdgeWithoutLabel {
            source_id: "a".to_owned(),
            target_id: "b".to_owned()
        }
    );
    let truncated = parse_graphml("<graphml><graph><node id=\"a\">").unwrap_err();
    assert!(
        matches!(truncated, GraphmlError::UnexpectedEof { .. }),
        "{truncated}"
    );
    let malformed =
        parse_graphml("<graphml><graph><node id=\"a\"></nod></graph></graphml>").unwrap_err();
    assert!(matches!(malformed, GraphmlError::Xml { .. }), "{malformed}");
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn async_reader_yields_the_same_records() {
    use crate::read::AsyncGraphmlReader;
    let mut reader = AsyncGraphmlReader::new(NEO4J_EXPORT.as_bytes());
    let mut count = 0;
    while let Some(record) = reader.next_record().await.expect("parse") {
        if let GraphmlRecord::Edge(edge) = record {
            assert_eq!(edge.label, "KNOWS");
        }
        count += 1;
    }
    assert_eq!(count, 3);
}
