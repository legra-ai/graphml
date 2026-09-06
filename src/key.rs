//! `<key>` declarations: the schema every `<data>` element resolves
//! against.

use std::collections::HashMap;

/// Which elements a `<key>` applies to (`for` attribute).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyDomain {
    /// `for="node"`.
    Node,
    /// `for="edge"`.
    Edge,
    /// `for="graph"`, `for="graphml"`, `for="all"`, or absent.
    Other,
}

impl KeyDomain {
    fn parse(raw: Option<&str>) -> Self {
        match raw {
            Some("node") => Self::Node,
            Some("edge") => Self::Edge,
            _ => Self::Other,
        }
    }
}

/// The declared `attr.type` of a `<key>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    /// `boolean`
    Boolean,
    /// `int`
    Int,
    /// `long`
    Long,
    /// `float`
    Float,
    /// `double`
    Double,
    /// `string`
    String,
}

impl KeyType {
    fn parse(raw: &str) -> Option<Self> {
        Some(match raw {
            "boolean" => Self::Boolean,
            "int" => Self::Int,
            "long" => Self::Long,
            "float" => Self::Float,
            "double" => Self::Double,
            "string" => Self::String,
            _ => return None,
        })
    }

    /// The GraphML spelling, used as the property type hint.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Int => "int",
            Self::Long => "long",
            Self::Float => "float",
            Self::Double => "double",
            Self::String => "string",
        }
    }
}

/// One `<key>` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KeyDecl {
    pub(crate) name: String,
    pub(crate) domain: KeyDomain,
    pub(crate) attr_type: Option<KeyType>,
}

impl KeyDecl {
    pub(crate) fn new(
        id: &str,
        name: Option<&str>,
        domain: Option<&str>,
        attr_type: Option<&str>,
    ) -> Self {
        Self {
            name: name.unwrap_or(id).to_owned(),
            domain: KeyDomain::parse(domain),
            attr_type: attr_type.and_then(KeyType::parse),
        }
    }

    /// Whether this key carries the element's label(s). Exporters spell
    /// it `label`, `labels`, or `:LABEL`-style with a leading colon.
    pub(crate) fn is_label(&self) -> bool {
        let name = self.name.trim_start_matches(':');
        name.eq_ignore_ascii_case("label") || name.eq_ignore_ascii_case("labels")
    }
}

/// The declared keys by id.
#[derive(Debug, Default)]
pub(crate) struct Keys {
    decls: HashMap<String, KeyDecl>,
}

impl Keys {
    pub(crate) fn insert(&mut self, id: String, decl: KeyDecl) {
        self.decls.insert(id, decl);
    }

    pub(crate) fn get(&self, id: &str) -> Option<&KeyDecl> {
        self.decls.get(id)
    }
}

/// Split a label value into labels: Neo4j writes `:Person:Employee`,
/// others `Person;Employee` or `Person,Employee`.
pub(crate) fn split_labels(raw: &str) -> Vec<String> {
    raw.split([':', ';', ','])
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_declarations_default_name_to_id_and_classify_labels() {
        let key = KeyDecl::new("d0", None, Some("node"), Some("int"));
        assert_eq!(key.name, "d0");
        assert_eq!(key.domain, KeyDomain::Node);
        assert_eq!(key.attr_type, Some(KeyType::Int));
        assert!(!key.is_label());
        assert!(KeyDecl::new("l", Some("labels"), Some("node"), None).is_label());
        assert!(KeyDecl::new("l", Some(":LABEL"), None, None).is_label());
    }

    #[test]
    fn labels_split_on_every_common_separator() {
        assert_eq!(split_labels(":Person:Employee"), ["Person", "Employee"]);
        assert_eq!(split_labels("Person; Employee"), ["Person", "Employee"]);
        assert_eq!(split_labels("Person"), ["Person"]);
        assert_eq!(split_labels("").len(), 0);
    }
}
