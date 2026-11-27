use crate::fs::FsError;
use crate::kprint;
use crate::spinlock::SpinLock;
use crate::{console, fs};

static CWD: SpinLock<&'static fs::Node> = SpinLock::new(&fs::hierarchy::ROOT);

pub fn cwd() -> &'static fs::Node {
    *CWD.lock()
}

pub fn help() {
    crate::kprintln!("help: list available commands");
    crate::kprintln!("ls [PATH ...]: list files or directories (default: current directory)");
    crate::kprintln!("cat FILE ...: print file contents");
    crate::kprintln!("cd [PATH]: change directory (default: /)");
    crate::kprintln!("pwd: print the current directory");
    crate::kprintln!("clear: clear the terminal");
}

pub fn clear() {
    kprint!("\x1b[2J\x1b[H");
}

pub fn pwd() {
    crate::kprintln!("{}", fs::NodePath(cwd()));
}

pub fn cd<'a>(args: impl Iterator<Item = &'a str>) {
    let mut args = args.peekable();
    let first = args.next().unwrap_or("/");
    let multiple = args.peek().is_some();
    if multiple {
        crate::kprintln!("cd: One argument expected. Multiple given");
        return;
    }
    if let Err(error) = move_to(first) {
        report_error("cd", first, error);
    }
}

fn move_to(path: &str) -> Result<(), FsError> {
    let node = resolve_path(path)?;
    node.children()?;
    *CWD.lock() = node;
    Ok(())
}

pub fn ls<'a>(args: impl Iterator<Item = &'a str>) {
    let mut args = args.peekable();
    if args.peek().is_none() {
        list_path(".");
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
    fs::lookup_from(cwd(), path)
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
