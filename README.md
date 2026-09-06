# graphml

[![Crates.io](https://img.shields.io/crates/v/graphml.svg)](https://crates.io/crates/graphml)
[![Downloads](https://img.shields.io/crates/d/graphml.svg)](https://crates.io/crates/graphml)
[![Documentation](https://docs.rs/graphml/badge.svg)](https://docs.rs/graphml)
[![CI](https://github.com/legra-ai/graphml/actions/workflows/ci.yml/badge.svg)](https://github.com/legra-ai/graphml/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/legra-ai/graphml)

A streaming [GraphML](http://graphml.graphdrawing.org/) parser that yields
owned property-graph nodes and edges one element at a time.

- `<key>` declarations are honoured: every `<data>` resolves to its key's
  `attr.name` and `attr.type` (kept as a type hint), and an undeclared key
  is an error rather than a silent drop.
- Labels come from a key named `label`/`labels` (Neo4j's `:Person:Employee`
  spelling and `;`/`,` separators are all split), or from an edge's `label`
  attribute.
- Hierarchical GraphML (a `<graph>` nested in a `<node>`) is tolerated: the
  outer node is emitted, nested content is skipped.
- Memory is bounded by the key table plus one open element, however large
  the document. Built on `quick-xml`; a `tokio` feature adds
  `AsyncGraphmlReader` over any `AsyncBufRead`.

The crate depends on no database driver, graph store, RDF model, or
application framework. The optional `property-graph-model` feature adds
field-for-field `From` conversions into that crate's `PgNode` / `PgEdge`.

## Quick start

```rust
use graphml::parse_graphml;

let input = r#"<graphml>
  <key id="labels" for="node" attr.name="labels"/>
  <key id="age" for="node" attr.name="age" attr.type="int"/>
  <graph edgedefault="directed">
    <node id="alice"><data key="labels">:Person</data><data key="age">30</data></node>
    <node id="bob"><data key="labels">:Person</data></node>
    <edge source="alice" target="bob" label="KNOWS"/>
  </graph>
</graphml>"#;

let (nodes, edges) = parse_graphml(input)?;
assert_eq!(nodes[0].id, "alice");
assert_eq!(nodes[0].labels, ["Person"]);
assert_eq!(nodes[0].properties[0].name, "age");
assert_eq!(nodes[0].properties[0].type_hint.as_deref(), Some("int"));
assert_eq!(edges[0].label, "KNOWS");
# Ok::<(), graphml::GraphmlError>(())
```

## Streaming large documents

```rust
use graphml::{GraphmlReader, GraphmlRecord};

let input = r#"<graphml><graph><node id="a"/><edge source="a" target="a" label="SELF"/></graph></graphml>"#;
let mut edges = 0;
for record in GraphmlReader::new(input.as_bytes()) {
    if let GraphmlRecord::Edge(_) = record? {
        edges += 1;
    }
}
assert_eq!(edges, 1);
# Ok::<(), graphml::GraphmlError>(())
```

With the `tokio` feature, `AsyncGraphmlReader::new(reader).next_record().await`
does the same over an `AsyncBufRead`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
