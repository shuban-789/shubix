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