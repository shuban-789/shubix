use crate::{console, fs};

pub fn help() {
    crate::kprintln!("help: list available commands");
    crate::kprintln!("ls [PATH ...]: list files or directories (default: /)");
    crate::kprintln!("cat FILE ...: print file contents");
}

pub fn ls<'a>(args: impl Iterator<Item = &'a str>) {
    let mut args = args.peekable();
    if args.peek().is_none() {
        list_path("/");
        return;
    }

    let first = args.next().unwrap();
    let multiple = args.peek().is_some();
    if multiple {
        crate::kprintln!("{}:", first);
    }
    list_path(first);
    for path in args {
        crate::kprintln!("\n{}:", path);
        list_path(path);
    }
}

fn list_path(path: &str) {
    match resolve_path(path) {
        Ok(node) => match &node.kind {
            fs::NodeKind::Directory(children) => {
                for child in *children {
                    crate::kprintln!("{}", child.name);
                }
            }
            fs::NodeKind::File(_) => crate::kprintln!("{}", path),
        },
        Err(error) => report_error("ls", path, error),
    }
}

pub fn cat<'a>(args: impl Iterator<Item = &'a str>) {
    let mut args = args.peekable();
    if args.peek().is_none() {
        crate::kprintln!("Usage: cat FILE ...");
        return;
    }

    for path in args {
        match resolve_path(path).and_then(|node| node.contents()) {
            Ok(bytes) => {
                for &byte in bytes {
                    console::putc(byte);
                }
            }
            Err(error) => report_error("cat", path, error),
        }
    }
}

fn resolve_path(path: &str) -> Result<&'static fs::Node, fs::FsError> {
    if path.starts_with('/') {
        return fs::lookup(path);
    }
    if path.is_empty() || path.len() > 128 {
        return Err(fs::FsError::InvalidPath);
    }

    let mut absolute = [0u8; 129];
    absolute[0] = b'/';
    absolute[1..path.len() + 1].copy_from_slice(path.as_bytes());
    let absolute =
        core::str::from_utf8(&absolute[..path.len() + 1]).map_err(|_| fs::FsError::InvalidPath)?;
    fs::lookup(absolute)
}

fn report_error(command: &str, path: &str, error: fs::FsError) {
    let message = match error {
        fs::FsError::InvalidPath => "invalid or unsupported path",
        fs::FsError::NotFound => "no such file or directory",
        fs::FsError::NotADirectory => "not a directory",
        fs::FsError::IsADirectory => "is a directory",
    };
    crate::kprintln!("{}: {}: {}", command, path, message);
}

pub fn invalid(command: &str) {
    crate::kprintln!("Unknown: {}", command);
}
