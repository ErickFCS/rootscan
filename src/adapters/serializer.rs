//! Output adapters implementing the serializer port.
//!
//! The YAML and JSON formats share the same tree shape:
//!
//! ```yaml
//! .:
//!   src:
//!     main.rs: {}
//!     lib.rs: {}
//!   Cargo.toml: {}
//! ```
//!
//! Files render as empty mappings; directories render as their own mapping.

use serde_json::{Map, Value};

use crate::domain::tree::{DirNode, Node, Tree};
use crate::error::{Error, Result};
use crate::ports::serializer::{OutputFormat, Serializer};

/// Returns the serializer matching the requested output format.
pub fn select(format: OutputFormat) -> &'static dyn Serializer {
    match format {
        OutputFormat::Yaml => &YamlSerializer,
        OutputFormat::Json => &JsonSerializer,
        OutputFormat::Text => &TextSerializer,
    }
}

/// Converts a tree into an ordered JSON-like value (insertion order is
/// preserved thanks to `serde_json`'s `preserve_order` feature, which the
/// YAML serializer also relies on).
fn to_value(dir: &DirNode) -> Map<String, Value> {
    let mut map = Map::new();
    for (name, node) in &dir.children {
        let value = match node {
            Node::File => Map::new(),
            Node::Dir(sub) => to_value(sub),
        };
        map.insert(name.clone(), Value::Object(value));
    }
    map
}

fn document(tree: &Tree) -> Value {
    Value::Object(Map::from_iter([(tree.name.clone(), Value::Object(to_value(&tree.root)))]))
}

struct YamlSerializer;

impl Serializer for YamlSerializer {
    fn serialize(&self, tree: &Tree) -> Result<Vec<u8>> {
        let rendered = serde_yaml::to_string(&document(tree))
            .map_err(|err| Error::Serialize("yaml", Box::new(err)))?;
        Ok(rendered.into_bytes())
    }
}

struct JsonSerializer;

impl Serializer for JsonSerializer {
    fn serialize(&self, tree: &Tree) -> Result<Vec<u8>> {
        let mut bytes = serde_json::to_string_pretty(&document(tree))
            .map_err(|err| Error::Serialize("json", Box::new(err)))?
            .into_bytes();
        bytes.push(b'\n');
        Ok(bytes)
    }
}

struct TextSerializer;

impl Serializer for TextSerializer {
    fn serialize(&self, tree: &Tree) -> Result<Vec<u8>> {
        let mut lines = Vec::new();
        walk(&tree.root, "", &mut lines);
        let mut bytes = lines.join("\n").into_bytes();
        if bytes.is_empty() {
            bytes.push(b'\n');
        }
        bytes.push(b'\n');
        Ok(bytes)
    }
}

/// Depth-first emission of one path per line in discovery/sorted order.
fn walk(dir: &DirNode, prefix: &str, lines: &mut Vec<String>) {
    for (name, node) in &dir.children {
        match node {
            Node::File => lines.push(format!("{prefix}{name}")),
            Node::Dir(sub) => {
                let child_prefix = format!("{prefix}{name}/");
                lines.push(child_prefix.clone());
                walk(sub, &child_prefix, lines);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{Entry, EntryKind};
    use crate::domain::tree::TreeOptions;
    use std::path::PathBuf;

    fn sample_tree() -> Tree {
        let entries = [
            Entry { relative: PathBuf::from("src/main.rs"), kind: EntryKind::File },
            Entry { relative: PathBuf::from("Cargo.toml"), kind: EntryKind::File },
        ];
        Tree::build("proj", &entries, TreeOptions { sorted: true, dirs_only: false })
    }

    #[test]
    fn yaml_renders_tree_shape() {
        let bytes = select(OutputFormat::Yaml).serialize(&sample_tree()).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert_eq!(text, "proj:\n  src:\n    main.rs: {}\n  Cargo.toml: {}\n");
    }

    #[test]
    fn json_renders_same_shape() {
        let bytes = select(OutputFormat::Json).serialize(&sample_tree()).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["proj"]["src"]["main.rs"].as_object().unwrap().len(), 0);
        assert!(value["proj"]["Cargo.toml"].is_object());
    }

    #[test]
    fn text_lists_directories_with_trailing_slash() {
        let bytes = select(OutputFormat::Text).serialize(&sample_tree()).unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), "src/\nsrc/main.rs\nCargo.toml\n");
    }
}
