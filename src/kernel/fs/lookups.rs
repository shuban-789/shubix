use crate::fs::hierarchy::ROOT;
use crate::fs::structs::FsError;
use crate::fs::structs::Node;

pub fn lookup_child<'a>(directory: &'a Node, name: &str) -> Option<&'a Node> {
    directory
        .children()
        .ok()?
        .iter()
        .find(|child| child.name == name)
}

pub fn lookup(path: &str) -> Result<&'static Node, FsError> {
    if !path.starts_with('/') {
        return Err(FsError::InvalidPath);
    }
    lookup_from(&ROOT, path)
}

pub fn lookup_from(cwd: &'static Node, path: &str) -> Result<&'static Node, FsError> {
    if path.is_empty() || path.as_bytes().contains(&0) {
        return Err(FsError::InvalidPath);
    }

    let mut node = if path.starts_with('/') { &ROOT } else { cwd };
    for component in path.split('/').filter(|component| !component.is_empty()) {
        node.children()?;
        node = match component {
            "." => node,
            ".." => parent(&ROOT, node).unwrap_or(&ROOT),
            name => lookup_child(node, name).ok_or(FsError::NotFound)?,
        };
    }

    if path.ends_with('/') {
        node.children()?;
    }
    Ok(node)
}

fn parent(directory: &'static Node, node: &Node) -> Option<&'static Node> {
    for child in directory.children().ok()? {
        if core::ptr::eq(child, node) {
            return Some(directory);
        }
        if let Some(found) = parent(child, node) {
            return Some(found);
        }
    }
    None
}

pub struct NodePath(pub &'static Node);

impl core::fmt::Display for NodePath {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if core::ptr::eq(self.0, &ROOT) {
            return f.write_str("/");
        }
        if let Some(directory) = parent(&ROOT, self.0) {
            if !core::ptr::eq(directory, &ROOT) {
                write!(f, "{}", NodePath(directory))?;
            }
        }
        write!(f, "/{}", self.0.name)
    }
}
