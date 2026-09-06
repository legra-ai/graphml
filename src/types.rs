//! Owned records produced by the parser.

/// A GraphML `<node>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The `id` attribute.
    pub id: String,
    /// Labels from the `label`/`labels` data key (split on `:`, `;`, `,`).
    pub labels: Vec<String>,
    /// `<data>` values in document order, excluding the label key.
    pub properties: Vec<Property>,
}

/// A GraphML `<edge>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// The `id` attribute, when present.
    pub id: Option<String>,
    /// The `source` attribute.
    pub source: String,
    /// The `target` attribute.
    pub target: String,
    /// The edge label: the `label` attribute, or the `label`/`labels`
    /// data key.
    pub label: String,
    /// `<data>` values in document order, excluding the label key.
    pub properties: Vec<Property>,
}

/// One `<data>` value with the declared key's name and type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    /// The key's `attr.name` (falling back to the key `id`).
    pub name: String,
    /// The text content, XML-unescaped.
    pub value: String,
    /// The key's `attr.type` (`boolean`, `int`, `long`, `float`,
    /// `double`, `string`), when declared.
    pub type_hint: Option<String>,
}
