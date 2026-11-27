//! Reader and writer for ChurchPresenter song files.
use crate::discovery::{extension, normalized_roots, walk};
use crate::quick_play::{leading_number, number_prefix};
use cp_core::{SongItem, SongTranslation};
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::LazyLock;

static LANGUAGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^\[\s*(?:(primary)|(secondary)|translation\s+([0-9]+))\s*(?::\s*([^\]]*))?\s*\]$",
    )
    .unwrap()
});

/// A library song with the additional header fields needed for lossless metadata editing.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SongFile {
    #[serde(flatten)]
    pub song: SongItem,
    #[serde(default, rename = "extraHeader")]
    pub extra_header: IndexMap<String, String>,
}

impl Deref for SongFile {
    type Target = SongItem;
    fn deref(&self) -> &SongItem {
        &self.song
    }
}

impl DerefMut for SongFile {
    fn deref_mut(&mut self) -> &mut SongItem {
        &mut self.song
    }
}

impl From<SongItem> for SongFile {
    fn from(song: SongItem) -> Self {
        Self {
            song,
            ..Self::default()
        }
    }
}

/// Reads UTF-8 song files and older Cyrillic libraries encoded in Windows-1251.
pub fn parse_song_file(path: impl AsRef<Path>, songbook: &str) -> io::Result<SongFile> {
    let bytes = fs::read(&path)?;
    let unmarked = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes);
    let text = match std::str::from_utf8(unmarked) {
        Ok(s) if !s.contains('\u{fffd}') => Cow::Borrowed(s),
        _ => {
            encoding_rs::WINDOWS_1251
                .decode_without_bom_handling(&bytes)
                .0
        }
    };
    Ok(parse_song_content(&text, path, songbook))
}

fn trim_blank_lines(lines: &mut Vec<String>) {
    let first = lines
        .iter()
        .position(|line| !line.trim().is_empty())
        .unwrap_or(lines.len());
    let end = lines
        .iter()
        .rposition(|line| !line.trim().is_empty())
        .map_or(first, |i| i + 1);
    lines.truncate(end);
    lines.drain(..first);
}

/// Parses song text using the filename for the hymn number and fallback title.
pub fn parse_song_content(content: &str, path: impl AsRef<Path>, songbook: &str) -> SongFile {
    let path = path.as_ref();
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let digits = number_prefix(&stem);
    let number = if stem.starts_with(digits)
        && !digits.is_empty()
        && stem[digits.len()..].trim_start().starts_with('-')
    {
        digits
    } else {
        leading_number(&stem).unwrap_or_default()
    };
    let mut song = SongItem {
        number: number.to_owned(),
        songbook: songbook.to_owned(),
        source_file: path.to_string_lossy().into_owned(),
        ..Default::default()
    };
    let content = content
        .trim_start_matches('\u{feff}')
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let lines: Vec<_> = content.lines().collect();
    let mut extra_header = IndexMap::new();
    let mut body_start = 0;
    if let Some(start) = lines.iter().position(|line| line.trim() == "---") {
        let after = &lines[start + 1..];
        let close = after.iter().position(|line| line.trim() == "---");
        if let Some(end) = close {
            body_start = start + 2 + end;
        }
        for line in &after[..close.unwrap_or(after.len())] {
            if let Some((key, value)) = line.trim().split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim().to_owned();
                match key.as_str() {
                    "author" => song.author = value,
                    "composer" => song.composer = value,
                    "tune" => song.tune = value,
                    "ccli" => song.ccli_number = value,
                    "" => {}
                    _ => {
                        extra_header.insert(key, value);
                    }
                }
            }
        }
    }

    let mut languages: [Option<SongTranslation>; 4] = Default::default();
    let mut active: Option<usize> = None;
    for line in &lines[body_start..] {
        let trimmed = line.trim();
        if let Some(tag) = LANGUAGE.captures(trimmed) {
            active = if tag.get(1).is_some() {
                Some(0)
            } else if tag.get(2).is_some() {
                Some(1)
            } else {
                tag[3]
                    .parse::<usize>()
                    .ok()
                    .filter(|n| (3..=4).contains(n))
                    .map(|n| n - 1)
            };
            if let Some(index) = active {
                languages[index].get_or_insert_with(|| SongTranslation {
                    label: tag
                        .get(4)
                        .map_or("", |label| label.as_str().trim())
                        .to_owned(),
                    ..Default::default()
                });
            }
        } else if let Some(half) = active.and_then(|index| languages[index].as_mut()) {
            if trimmed
                .get(..6)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("title:"))
            {
                half.title = trimmed[6..].trim().to_owned();
            } else {
                half.lyrics.push((*line).to_owned());
            }
        }
    }
    for half in languages.iter_mut().flatten() {
        trim_blank_lines(&mut half.lyrics);
    }
    let primary = languages[0].take().unwrap_or_default();
    song.title = if primary.title.is_empty() {
        stem.into_owned()
    } else {
        primary.title
    };
    song.lyrics = primary.lyrics;
    song.translations = languages
        .into_iter()
        .flatten()
        .filter(|half| !half.is_empty())
        .collect();
    if let Some(second) = song.translations.first() {
        song.secondary_title.clone_from(&second.title);
        song.secondary_lyrics.clone_from(&second.lyrics);
    }
    SongFile { song, extra_header }
}

/// Writes a UTF-8 song file, replacing the destination once the complete song is written.
pub fn write_song_file(song: &SongFile, path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let mut out = tempfile::NamedTempFile::new_in(parent)?;
    let credits = [
        ("author", &song.author),
        ("composer", &song.composer),
        ("tune", &song.tune),
        ("ccli", &song.ccli_number),
    ];
    if credits.iter().any(|(_, v)| !v.is_empty()) || !song.extra_header.is_empty() {
        writeln!(out, "---")?;
        for (key, value) in credits {
            if !value.is_empty() {
                writeln!(out, "{key}: {value}")?;
            }
        }
        for (key, value) in &song.extra_header {
            writeln!(out, "{key}: {value}")?;
        }
        writeln!(out, "---\n")?;
    }
    writeln!(out, "[Primary]\ntitle: {}\n", song.title)?;
    for line in &song.lyrics {
        writeln!(out, "{line}")?;
    }

    let legacy;
    let translations = if song.translations.is_empty() {
        legacy = [SongTranslation {
            label: String::new(),
            title: song.secondary_title.clone(),
            lyrics: song.secondary_lyrics.clone(),
        }];
        legacy.as_slice()
    } else {
        &song.translations
    };
    for (index, half) in translations
        .iter()
        .take(3)
        .enumerate()
        .filter(|(_, t)| !t.is_empty())
    {
        let tag = if index == 0 {
            "Secondary".to_owned()
        } else {
            format!("Translation {}", index + 2)
        };
        if half.label.trim().is_empty() {
            writeln!(out, "\n[{tag}]")?;
        } else {
            writeln!(out, "\n[{tag}: {}]", half.label)?;
        }
        if !half.title.is_empty() {
            writeln!(out, "title: {}", half.title)?;
        }
        writeln!(out)?;
        for line in &half.lyrics {
            writeln!(out, "{line}")?;
        }
    }
    out.flush()?;
    out.persist(path)?;
    Ok(())
}

/// Loads files in root order. Songbook paths use '/' for nested directories on every platform.
pub fn load_songs_from_directories(roots: &[impl AsRef<Path>]) -> Vec<SongFile> {
    let mut seen = BTreeSet::new();
    let mut songs = Vec::new();
    for root in normalized_roots(roots) {
        if !root.is_dir() {
            continue;
        }
        for file in walk(&root) {
            let path = file.path();
            if extension(path) != "song" || !seen.insert(path.to_owned()) {
                continue;
            }
            let parent = path
                .parent()
                .and_then(|p| p.strip_prefix(&root).ok())
                .unwrap_or(Path::new(""));
            let songbook = parent
                .components()
                .map(|p| p.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            if let Ok(song) = parse_song_file(path, &songbook) {
                songs.push(song);
            }
        }
    }
    songs
}
