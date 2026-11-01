//! Builds the tree-shaped domain structure out of a flat entry list.
//!
//! Parent directories may be listed implicitly; this module materializes
//! them. Name uniqueness per directory is assumed (the filesystem
//! guarantees it). Leaf names live in the map keys, so file nodes carry no
//! payload by design.

use indexmap::IndexMap;

use crate::domain::model::{Entry, EntryKind};

/// Options controlling tree construction.
#[derive(Debug, Clone, Copy)]
pub struct TreeOptions {
    /// Sort every directory's children: directories first, then files,
    /// both case-insensitively. `false` keeps the scan's discovery order.
    pub sorted: bool,
    /// Omit file entries, listing only directories.
    pub dirs_only: bool,
}

#[derive(Debug, Default)]
pub struct DirNode {
    /// Children in display order (discovery or sorted).
    pub children: IndexMap<String, Node>,
}

#[derive(Debug)]
pub enum Node {
    File,
    Dir(DirNode),
}

#[derive(Debug)]
pub struct Tree {
    /// Display name of the scan root (usually the last path component).
    pub name: String,
    pub root: DirNode,
}

impl Tree {
    /// Builds a tree from a flat entry list.
    pub fn build(name: impl Into<String>, entries: &[Entry], options: TreeOptions) -> Tree {
        let mut root = DirNode::default();
        for entry in entries {
            Self::insert_into(&mut root, &entry.relative, entry.kind, options.dirs_only);
        }
        if options.sorted {
            let root = Self::sorted(root);
            return Tree { name: name.into(), root };
        }
        Tree { name: name.into(), root }
    }

    /// Inserts one relative path, creating intermediate directories and
    /// tolerating duplicate directory notifications.
    fn insert_into(dir: &mut DirNode, relative: &std::path::Path, kind: EntryKind, dirs_only: bool) {
        let mut components = relative.components();
        let mut cursor = dir;

        // All components except the last one must end up directories.
        if let Some(parents) = components.next_back() {
            for component in components {
                let name = component.as_os_str().to_string_lossy().into_owned();
                cursor = dir_child(cursor, name);
            }
            let name = parents.as_os_str().to_string_lossy().into_owned();
            match kind {
                EntryKind::Directory => {
                    dir_child(cursor, name);
                }
                EntryKind::File => {
                    if !dirs_only {
                        cursor.children.entry(name).or_insert(Node::File);
                    }
                }
            }
        }
    }

    /// Recursively rebuilds each directory's children in sorted order.
    fn sorted(dir: DirNode) -> DirNode {
        let mut items: Vec<(String, Node)> = dir.children.into_iter().collect();
        items.sort_by(|(a, node_a), (b, node_b)| {
            let rank_a = dir_rank(node_a);
            let rank_b = dir_rank(node_b);
            rank_a.cmp(&rank_b).then_with(|| a.to_lowercase().cmp(&b.to_lowercase())).then_with(|| a.cmp(b))
        });
        DirNode {
            children: items
                .into_iter()
                .map(|(name, node)| {
                    let node = match node {
                        Node::Dir(sub) => Node::Dir(Self::sorted(sub)),
                        file => file,
                    };
                    (name, node)
                })
                .collect(),
        }
    }
}

/// Directory/file sort rank: directories come first.
fn dir_rank(node: &Node) -> u8 {
    match node {
        Node::Dir(_) => 0,
        Node::File => 1,
    }
}

/// Returns the child directory with the given name, creating it if needed,
/// or descending into the existing one. A stale file node in that slot is
/// replaced (cannot happen in practice on a real filesystem).
fn dir_child(dir: &mut DirNode, name: String) -> &mut DirNode {
    if !matches!(dir.children.get(&name), Some(Node::Dir(_))) {
        dir.children.insert(name.clone(), Node::Dir(DirNode::default()));
    }
    match dir.children.get_mut(&name) {
        Some(Node::Dir(sub)) => sub,
        _ => unreachable!("just ensured a directory exists in this slot"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn entry(path: &str, kind: EntryKind) -> Entry {
        Entry { relative: PathBuf::from(path), kind }
    }

    fn names(node: &DirNode) -> Vec<&str> {
        node.children.keys().map(String::as_str).collect()
    }

    #[test]
    fn materializes_implicit_parents() {
        let entries = [entry("a/b/c.rs", EntryKind::File)];
        let tree = Tree::build("root", &entries, TreeOptions { sorted: false, dirs_only: false });

        let a = match &tree.root.children["a"] {
            Node::Dir(dir) => dir,
            Node::File => panic!("expected directory"),
        };
        assert!(matches!(a.children["b"], Node::Dir(_)));
    }

    #[test]
    fn duplicate_directories_are_deduplicated() {
        let entries = [
            entry("src", EntryKind::Directory),
            entry("src/main.rs", EntryKind::File),
        ];
        let tree = Tree::build("root", &entries, TreeOptions { sorted: false, dirs_only: false });
        assert_eq!(names(&tree.root), ["src"]);

        let src = match &tree.root.children["src"] {
            Node::Dir(dir) => dir,
            Node::File => panic!("expected directory"),
        };
        assert_eq!(names(src), ["main.rs"]);
    }

    #[test]
    fn sorted_puts_directories_first_case_insensitive() {
        let entries = [
            entry("zeta.rs", EntryKind::File),
            entry("README.md", EntryKind::File),
            entry("alpha", EntryKind::Directory),
            entry("Beta", EntryKind::Directory),
        ];
        let tree = Tree::build("root", &entries, TreeOptions { sorted: true, dirs_only: false });
        assert_eq!(names(&tree.root), ["alpha", "Beta", "README.md", "zeta.rs"]);
    }

    #[test]
    fn unsorted_keeps_discovery_order() {
        let entries = [
            entry("zeta.rs", EntryKind::File),
            entry("alpha", EntryKind::Directory),
        ];
        let tree = Tree::build("root", &entries, TreeOptions { sorted: false, dirs_only: false });
        assert_eq!(names(&tree.root), ["zeta.rs", "alpha"]);
    }

    #[test]
    fn dirs_only_omits_files_but_keeps_directories() {
        let entries = [
            entry("src/main.rs", EntryKind::File),
            entry("src/sub", EntryKind::Directory),
            entry("top.rs", EntryKind::File),
        ];
        let tree = Tree::build("root", &entries, TreeOptions { sorted: false, dirs_only: true });
        assert_eq!(names(&tree.root), ["src"]);

        let src = match &tree.root.children["src"] {
            Node::Dir(dir) => dir,
            Node::File => panic!("expected directory"),
        };
        assert_eq!(names(src), ["sub"]);
    }
}
