//! Hymnbook, hymn number and preferred-format selection.
use crate::discovery::{absolute, extension, normalized_roots, relative_to_root};
use crate::pinyin_match;
use indexmap::IndexMap;
use std::path::{Path, PathBuf};

/// Returns the first directory below the first configured root that contains the file.
pub fn book(file: &Path, roots: &[impl AsRef<Path>]) -> String {
    book_for_path(&absolute(file), &normalized_roots(roots))
}

fn book_for_path(path: &Path, roots: &[PathBuf]) -> String {
    roots
        .iter()
        .find_map(|root| relative_to_root(path, root))
        .and_then(Path::parent)
        .and_then(|parent| parent.components().next())
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub(crate) fn number_prefix(name: &str) -> &str {
    let trimmed = name.trim_start();
    &trimmed[..trimmed.bytes().take_while(u8::is_ascii_digit).count()]
}

/// Returns the leading decimal number without padding. An all-zero prefix yields "0".
pub fn leading_number(name: &str) -> Option<&str> {
    let digits = number_prefix(name);
    if digits.is_empty() {
        return None;
    }
    let significant = digits.trim_start_matches('0');
    Some(if significant.is_empty() {
        "0"
    } else {
        significant
    })
}

fn format_rank(extension: &str, preferred: &str) -> u8 {
    match (preferred, extension) {
        (p, e) if p == e => 0,
        ("pptx", "ppt" | "pptm" | "ppsx" | "pps") | ("docx", "doc") => 0,
        (_, "pptx") => 1,
        (_, "pdf") => 2,
        _ => 3,
    }
}

/// Selects one format per hymnbook and title. An empty query lists the selected book.
pub fn find(
    files: &[PathBuf],
    roots: &[impl AsRef<Path>],
    query: &str,
    preferred_book: &str,
    format: &str,
) -> Vec<PathBuf> {
    let text = query.trim().to_lowercase();
    let numeric = text.parse::<i32>().ok();
    let format = format.to_ascii_lowercase();
    let roots = normalized_roots(roots);
    let mut groups = IndexMap::new();
    for file in files {
        let path = absolute(file);
        let owner = book_for_path(&path, &roots);
        if !preferred_book.is_empty() && owner != preferred_book {
            continue;
        }
        let stem = file.file_stem().unwrap_or_default().to_string_lossy();
        let name = stem.to_lowercase();
        let matches = match numeric {
            Some(n) => {
                leading_number(&stem).and_then(|digits| digits.parse::<i32>().ok()) == Some(n)
            }
            None => name.contains(&text) || pinyin_match::matches(&stem, &text),
        };
        if matches {
            let rank = (format_rank(&extension(file), &format), path);
            let entry = groups
                .entry((owner, name))
                .or_insert_with(|| (rank.clone(), file));
            if rank < entry.0 {
                *entry = (rank, file);
            }
        }
    }
    let mut selected: Vec<_> = groups
        .into_iter()
        .map(|((_, name), (_, path))| (name != text, path))
        .collect();
    selected.sort_by(|(a, p), (b, q)| a.cmp(b).then_with(|| p.file_name().cmp(&q.file_name())));
    selected.into_iter().map(|(_, path)| path.clone()).collect()
}
