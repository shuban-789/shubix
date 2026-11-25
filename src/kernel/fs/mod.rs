//! A small, read-only filesystem compiled into the kernel.
//!
//! `ROOT` is ready to use at boot; no initialization or heap allocation is needed.
//! Add initial files to the arrays below. Writes, disk storage, permissions, and
//! per-process file descriptors can be added in later steps.

pub struct Node {
    pub name: &'static str,
    pub kind: NodeKind,
}

pub enum NodeKind {
    File(&'static [u8]),
    Directory(&'static [Node]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsError {
    InvalidPath,
    NotFound,
    NotADirectory,
    IsADirectory,
}

impl Node {
    pub fn children(&self) -> Result<&'static [Node], FsError> {
        match &self.kind {
            NodeKind::Directory(children) => Ok(children),
            NodeKind::File(_) => Err(FsError::NotADirectory),
        }
    }

    pub fn contents(&self) -> Result<&'static [u8], FsError> {
        match &self.kind {
            NodeKind::File(data) => Ok(data),
            NodeKind::Directory(_) => Err(FsError::IsADirectory),
        }
    }

    pub fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<usize, FsError> {
        let data = self.contents()?;
        if offset >= data.len() {
            return Ok(0);
        }

        let count = buffer.len().min(data.len() - offset);
        buffer[..count].copy_from_slice(&data[offset..offset + count]);
        Ok(count)
    }
}

pub fn lookup_child<'a>(directory: &'a Node, name: &str) -> Option<&'a Node> {
    directory
        .children()
        .ok()?
        .iter()
        .find(|child| child.name == name)
}

pub fn lookup(path: &str) -> Result<&'static Node, FsError> {
    if !path.starts_with('/')
        || path.as_bytes().contains(&0)
        || path.split('/').any(|component| component == "..")
    {
        return Err(FsError::InvalidPath);
    }

    let mut node = &ROOT;
    for component in path.split('/').filter(|component| !component.is_empty()) {
        node.children()?;
        if component != "." {
            node = lookup_child(node, component).ok_or(FsError::NotFound)?;
        }
    }

    if path.ends_with('/') {
        node.children()?;
    }
    Ok(node)
}

// fs tree initialization should happen here
static NOTES_CHILDREN: [Node; 1] = [Node {
    name: "todo.txt",
    kind: NodeKind::File(b"67\n"),
}];

static ROOT_CHILDREN: [Node; 2] = [
    Node {
        name: "hello.txt",
        kind: NodeKind::File(b"hello 67\n"),
    },
    Node {
        name: "notes",
        kind: NodeKind::Directory(&NOTES_CHILDREN),
    },
];

pub static ROOT: Node = Node {
    name: "/",
    kind: NodeKind::Directory(&ROOT_CHILDREN),
};
