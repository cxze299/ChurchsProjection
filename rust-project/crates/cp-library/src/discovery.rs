//! Presentation library traversal and path ownership.
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

const IGNORED: &[&str] = &[
    "node_modules",
    "dist",
    "build",
    ".git",
    "target",
    "$recycle.bin",
];
pub const EXTENSIONS: &[&str] = &[
    "pptx", "ppt", "ppsx", "pps", "pdf", "docx", "doc", "song", "png", "jpg", "jpeg",
];

pub(crate) fn extension(path: &Path) -> String {
    path.extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_ascii_lowercase()
}

pub(crate) fn absolute(path: &Path) -> PathBuf {
    let path = std::path::absolute(path).unwrap_or_else(|_| path.to_owned());
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            _ => result.push(part.as_os_str()),
        }
    }
    result
}

pub(crate) fn relative_to_root<'a>(path: &'a Path, root: &Path) -> Option<&'a Path> {
    let mut parts = path.components();
    for base in root.components() {
        let candidate = parts.next()?;
        let same = if cfg!(windows) {
            candidate.as_os_str().eq_ignore_ascii_case(base.as_os_str())
        } else {
            candidate == base
        };
        if !same {
            return None;
        }
    }
    Some(parts.as_path())
}

pub(crate) fn normalized_roots(roots: &[impl AsRef<Path>]) -> Vec<PathBuf> {
    let mut unique: Vec<PathBuf> = Vec::new();
    for root in roots {
        let root = root.as_ref();
        if root.as_os_str().is_empty() { continue; }
        let path = absolute(root);
        if !unique.iter().any(|p| relative_to_root(&path, p).is_some_and(|suffix| suffix.as_os_str().is_empty())) {
            unique.push(path);
        }
    }
    unique
}

pub(crate) fn walk(root: &Path) -> impl Iterator<Item = walkdir::DirEntry> {
    WalkDir::new(root)
        .follow_links(false)
        .sort_by(|a, b| {
            // Kotlin's song loader visits the files in each directory before its subdirectories.
            a.file_type()
                .is_dir()
                .cmp(&b.file_type().is_dir())
                .then_with(|| a.file_name().cmp(b.file_name()))
        })
        .into_iter()
        .filter_entry(|entry| {
            !entry.file_type().is_dir()
                || !IGNORED
                    .iter()
                    .any(|name| entry.file_name().eq_ignore_ascii_case(name))
        })
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_file() && !entry.file_name().to_string_lossy().starts_with("~$")
        })
}

/// Scans configured directories and returns unique absolute paths, newest first.
pub fn discover(roots: &[impl AsRef<Path>]) -> Vec<PathBuf> {
    let mut visited = BTreeSet::new();
    let mut found = Vec::new();
    for root in normalized_roots(roots) {
        if !root.is_dir() {
            continue;
        }
        for file in walk(&root) {
            let path = file.path();
            if EXTENSIONS.contains(&extension(path).as_str()) && visited.insert(path.to_owned()) {
                let modified = file
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                found.push((modified, file.into_path()));
            }
        }
    }
    found.sort_by(|(a, path_a), (b, path_b)| b.cmp(a).then_with(|| path_a.cmp(path_b)));
    found.into_iter().map(|(_, path)| path).collect()
}
