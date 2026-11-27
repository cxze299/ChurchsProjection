use crate::ENGLISH_BOOK_NAMES;
use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::Path;
use std::sync::LazyLock;

static BOOK_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([0-9]+)\s+(.+?)\s+([0-9]+)$").unwrap());
static VERSE_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(B[0-9]{3}C[0-9]{3}V[0-9]{3})\s+([0-9]+)\s+([0-9]+)\s+([0-9]+)\s+(.*)$").unwrap()
});
static ASIDE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\([^)]*\)").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibleBook {
    pub book_id: i32,
    pub name: String,
    pub chapter_count: i32,
    pub abbreviation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibleVerse {
    pub verse_id: String,
    pub book_id: i32,
    pub chapter: i32,
    pub verse_number: i32,
    pub verse_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibleSearch {
    pub book: String,
    pub chapter: i32,
    pub verse: i32,
    /// Display reference followed by the original verse text.
    pub verse_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchScope {
    All,
    Book(i32),
    Chapter(i32, i32),
}

fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn file_stem(path: &str) -> &str {
    let name = file_name(path);
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

pub fn extract_bible_abbreviation(title: Option<&str>, filename: &str) -> String {
    let title = title.unwrap_or_default();
    let without_asides = ASIDE.replace_all(title, " ");
    let stripped;
    let cleaned = if without_asides.trim().is_empty() {
        stripped = title
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>();
        stripped.trim()
    } else {
        without_asides.trim()
    };
    if cleaned.is_empty() {
        return file_stem(filename).to_owned();
    }
    let mut words = cleaned.split_whitespace();
    if let (Some(word), None) = (words.next(), words.next()) {
        let letters: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
        if !letters.is_empty() && letters.chars().count() <= 5 {
            return letters;
        }
    }
    cleaned
        .split_whitespace()
        .filter_map(|word| word.chars().find(|c| c.is_alphanumeric()))
        .take(4)
        .map(|c| c.to_uppercase().next().unwrap_or(c))
        .collect()
}

fn shorten_word(word: &str) -> String {
    if word.chars().count() <= 4 {
        word.to_owned()
    } else {
        word.chars().take(3).collect()
    }
}

pub fn generate_abbreviation(name: &str) -> String {
    let mut words = name.split_whitespace();
    let Some(first) = words.next() else {
        return String::new();
    };
    let Some(second) = words.next() else {
        return shorten_word(first);
    };
    if first.chars().all(char::is_numeric) {
        return format!("{first} {}", shorten_word(second));
    }
    if let Some(word) = name
        .split_whitespace()
        .find(|word| word.chars().count() > 3)
    {
        return shorten_word(word);
    }
    name.split_whitespace()
        .take(4)
        .filter_map(|word| word.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

pub fn parse_verse_code(code: &str) -> Option<(i32, i32, i32)> {
    let b = code.as_bytes();
    if b.len() != 12 || b[0] != b'B' || b[4] != b'C' || b[8] != b'V' {
        return None;
    }
    fn number(digits: &[u8]) -> Option<i32> {
        digits.iter().try_fold(0, |n, d| {
            d.is_ascii_digit().then(|| n * 10 + i32::from(d - b'0'))
        })
    }
    Some((number(&b[1..4])?, number(&b[5..8])?, number(&b[9..12])?))
}

type Chapter = (i32, i32);

#[derive(Debug, Clone, Copy)]
struct Location {
    chapter: Chapter,
    row: usize,
}

/// A translation indexed by its display chapters and canonical SPB codes.
/// Both indexes refer to the same stored verse text.
#[derive(Debug, Default)]
pub struct Bible {
    filename: String,
    title: String,
    abbreviation: String,
    books: Vec<BibleBook>,
    book_indices: HashMap<i32, usize>,
    chapter_index: HashMap<Chapter, Vec<BibleVerse>>,
    code_index: HashMap<String, Location>,
    code_to_display_map: HashMap<Chapter, Chapter>,
    file_order: Vec<Location>,
}

impl Bible {
    pub fn load_from_spb(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        Ok(Self::parse_spb(
            &fs::read_to_string(path)?,
            &path.to_string_lossy(),
        ))
    }

    pub fn parse_spb(text: &str, filename: &str) -> Self {
        Self::parse_spb_with_names(text, filename, &ENGLISH_BOOK_NAMES)
    }

    /// Supply localized canonical names for missing or placeholder book headers.
    pub fn parse_spb_with_names(text: &str, filename: &str, names: &[&str]) -> Self {
        let mut bible = Self {
            filename: file_name(filename).into(),
            ..Self::default()
        };
        let mut headers = Vec::new();
        let mut header_names = HashMap::new();
        let mut title = None;
        let mut in_header = true;
        let mut pending: Option<BibleVerse> = None;

        for line in text.trim_start_matches('\u{feff}').lines() {
            if let Some(name) = line.strip_prefix("##Title:") {
                title = Some(name.trim());
                continue;
            }
            if line.starts_with("##") {
                continue;
            }
            if line.starts_with("-----") {
                in_header = false;
                continue;
            }
            if in_header {
                if let Some(header) = BOOK_HEADER.captures(line) {
                    if let Ok(id) = header[1].parse::<i32>() {
                        if header_names
                            .insert(id, header[2].trim().to_owned())
                            .is_none()
                        {
                            headers.push(id);
                        }
                    }
                    continue;
                }
                if line.starts_with('B') {
                    in_header = false;
                }
            }

            if let Some(caps) = VERSE_LINE.captures(line) {
                let (Ok(book_id), Ok(chapter), Ok(verse_number)) =
                    (caps[2].parse(), caps[3].parse(), caps[4].parse())
                else {
                    continue;
                };
                if let Some(verse) = pending.take() {
                    bible.add_verse(verse);
                }
                let (code_book, code_chapter, _) = parse_verse_code(&caps[1]).unwrap();
                bible
                    .code_to_display_map
                    .insert((code_book, code_chapter), (book_id, chapter));
                bible.add_verse(BibleVerse {
                    verse_id: caps[1].into(),
                    book_id,
                    chapter,
                    verse_number,
                    verse_text: caps[5].trim().into(),
                });
            } else if let Some((book_id, chapter, verse_number)) = parse_verse_code(line) {
                if let Some(verse) = pending.take() {
                    bible.add_verse(verse);
                }
                pending = Some(BibleVerse {
                    verse_id: line.into(),
                    book_id,
                    chapter,
                    verse_number,
                    verse_text: String::new(),
                });
            } else if let Some(verse) = &mut pending {
                if !verse.verse_text.is_empty() {
                    verse.verse_text.push('\n');
                }
                verse.verse_text.push_str(line);
            }
        }
        if let Some(verse) = pending {
            bible.add_verse(verse);
        }

        bible.title = title.unwrap_or_else(|| file_stem(filename)).to_owned();
        bible.abbreviation = extract_bible_abbreviation(title, filename);
        bible.build_books(headers, header_names, names);
        bible
    }

    fn build_books(
        &mut self,
        mut headers: Vec<i32>,
        mut header_names: HashMap<i32, String>,
        names: &[&str],
    ) {
        let mut counts = BTreeMap::new();
        for &(id, chapter) in self.chapter_index.keys() {
            let count = counts.entry(id).or_insert(0);
            *count = chapter.max(*count);
        }
        headers.extend(
            counts
                .keys()
                .copied()
                .filter(|id| !header_names.contains_key(id)),
        );
        for id in headers {
            let header = header_names.remove(&id);
            let placeholder = header
                .as_deref()
                .and_then(|s| s.strip_prefix("Book "))
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()));
            let name = header
                .as_deref()
                .filter(|s| !s.is_empty() && !placeholder)
                .or_else(|| {
                    usize::try_from(id - 1)
                        .ok()
                        .and_then(|i| names.get(i).copied())
                })
                .or(header.as_deref())
                .map(str::to_owned)
                .unwrap_or_else(|| format!("Book {id}"));
            self.book_indices.insert(id, self.books.len());
            self.books.push(BibleBook {
                book_id: id,
                abbreviation: generate_abbreviation(&name),
                name,
                chapter_count: counts.get(&id).copied().unwrap_or(0),
            });
        }
    }

    fn add_verse(&mut self, mut verse: BibleVerse) {
        let chapter = (verse.book_id, verse.chapter);
        let rows = self.chapter_index.entry(chapter).or_default();
        let location = Location {
            chapter,
            row: rows.len(),
        };
        self.code_index.insert(verse.verse_id.clone(), location);
        self.file_order.push(location);
        if verse.verse_text.trim().len() != verse.verse_text.len() {
            verse.verse_text = verse.verse_text.trim().to_owned();
        }
        rows.push(verse);
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn abbreviation(&self) -> &str {
        &self.abbreviation
    }
    pub fn books(&self) -> &[BibleBook] {
        &self.books
    }
    pub fn book(&self, id: i32) -> Option<&BibleBook> {
        self.book_indices.get(&id).map(|&idx| &self.books[idx])
    }

    pub fn get_chapter_verses(&self, book_id: i32, chapter: i32) -> &[BibleVerse] {
        self.chapter_index
            .get(&(book_id, chapter))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Display rows retain file order. Adjacent parts of one numbered verse share one row.
    pub fn get_chapter(&self, book_id: i32, chapter: i32) -> Vec<BibleVerse> {
        let mut rows: Vec<BibleVerse> = Vec::new();
        for verse in self.get_chapter_verses(book_id, chapter) {
            if let Some(last) = rows
                .last_mut()
                .filter(|v| v.verse_number == verse.verse_number)
            {
                last.verse_id.push(',');
                last.verse_id.push_str(&verse.verse_id);
                if !last.verse_text.is_empty() && !verse.verse_text.is_empty() {
                    last.verse_text.push(' ');
                }
                last.verse_text.push_str(&verse.verse_text);
            } else {
                rows.push(verse.clone());
            }
        }
        rows
    }

    pub fn get_verse_details(&self, book_id: i32, chapter: i32, verse: i32) -> Option<&BibleVerse> {
        self.get_chapter_verses(book_id, chapter)
            .iter()
            .find(|v| v.verse_number == verse)
    }

    pub fn get_code_reference(&self, book_id: i32, chapter: i32, verse: i32) -> Option<&str> {
        self.get_verse_details(book_id, chapter, verse)
            .map(|v| v.verse_id.as_str())
    }

    fn at(&self, location: Location) -> &BibleVerse {
        &self.chapter_index[&location.chapter][location.row]
    }

    pub fn get_verse_details_by_code(&self, code: &str) -> Option<&BibleVerse> {
        if let Some(&location) = self.code_index.get(code) {
            return Some(self.at(location));
        }
        let (book, chapter, verse) = parse_verse_code(code)?;
        let (book, chapter) = self
            .code_to_display_map
            .get(&(book, chapter))
            .copied()
            .unwrap_or((book, chapter));
        self.get_verse_details(book, chapter, verse)
    }

    pub fn search_bible(&self, regex: &Regex, scope: SearchScope) -> Vec<BibleSearch> {
        self.search(scope, |text| regex.is_match(text))
    }

    /// Search for literal text ignoring case. In all-words mode, each whitespace-separated
    /// keyword must occur as a whole word, in any order.
    pub fn search_keywords(
        &self,
        query: &str,
        all_words: bool,
        scope: SearchScope,
    ) -> Vec<BibleSearch> {
        let query = query.trim();
        if query.is_empty() {
            return Vec::new();
        }
        if all_words {
            let patterns: Result<Vec<_>, _> = query
                .split_whitespace()
                .map(|word| Regex::new(&format!(r"(?i)\b{}\b", regex::escape(word))))
                .collect();
            return patterns
                .map(|words| self.search(scope, |s| words.iter().all(|word| word.is_match(s))))
                .unwrap_or_default();
        }
        regex::RegexBuilder::new(&regex::escape(query))
            .case_insensitive(true)
            .build()
            .map(|pattern| self.search_bible(&pattern, scope))
            .unwrap_or_default()
    }

    fn search(&self, scope: SearchScope, matches: impl Fn(&str) -> bool) -> Vec<BibleSearch> {
        let mut results = Vec::new();
        let mut visit = |v: &BibleVerse| {
            if matches(&v.verse_text) {
                let name = &self.book(v.book_id).expect("each verse has a book").name;
                results.push(BibleSearch {
                    book: name.clone(),
                    chapter: v.chapter,
                    verse: v.verse_number,
                    verse_text: format!(
                        "{} {}:{} {}",
                        name, v.chapter, v.verse_number, v.verse_text
                    ),
                });
            }
        };
        match scope {
            SearchScope::Chapter(id, chapter) => {
                self.get_chapter_verses(id, chapter).iter().for_each(visit)
            }
            _ => {
                for &location in &self.file_order {
                    if scope == SearchScope::All || scope == SearchScope::Book(location.chapter.0) {
                        visit(self.at(location));
                    }
                }
            }
        }
        results
    }
}
