use crate::initials::{candidates, CHINESE_BOOK_INITIALS};
use crate::names::{CHINESE_BOOK_NAMES, ENGLISH_ALIASES, ENGLISH_BOOK_NAMES};
use crate::Bible;
use regex::{Captures, Regex};
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmartReference {
    /// One-based book ID, independent of the module's display order.
    pub book_id: i32,
    pub chapter: Option<i32>,
    pub verse_start: Option<i32>,
    pub verse_end: Option<i32>,
}

const VERSE_SPAN: &str = r"(?:\s*[:.]\s*|\s+)([0-9]+)(?:(?:\s*-\s*|\s+)([0-9]+))?节?";
static REFERENCE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"^(.*?)\s*([0-9]+)(?:{VERSE_SPAN})?$")).unwrap());

// Each match consumes one complete passage from the front of the remaining input.
// The next book can begin immediately after the final digit of this passage.
static PASSAGE: LazyLock<Regex> = LazyLock::new(|| {
    let book = r"((?:[1-3]\s*)?\p{L}+(?:\s+\p{L}+)*)";
    Regex::new(&format!(r"^{book}\s+([0-9]+){VERSE_SPAN}")).unwrap()
});

fn normalized(input: &str) -> String {
    input
        .trim()
        .replace(['：', '，', ',', '章'], ":")
        .replace(['－', '–'], "-")
}

fn compact_name(name: &str) -> String {
    name.trim()
        .trim_end_matches('.')
        .split_whitespace()
        .collect::<String>()
        .to_lowercase()
}

fn book_for(token: &str, bible: Option<&Bible>, require_unique_name: bool) -> Option<i32> {
    let token = token.trim();
    if token.is_empty() || !token.chars().any(char::is_alphabetic) {
        return None;
    }
    let available = |id| bible.is_none_or(|b| b.book(id).is_some());
    let initials = candidates(token);
    if let Some(&id) = initials.first() {
        if CHINESE_BOOK_INITIALS[(id - 1) as usize].eq_ignore_ascii_case(token) {
            return available(id).then_some(id);
        }
    }
    let query = compact_name(token);
    let mut matches = Vec::new();
    let mut consider = |id, name: &str| {
        let name = compact_name(name);
        let rank = if name == query {
            0
        } else if name.starts_with(&query) {
            1
        } else if name.contains(&query) {
            2
        } else {
            return;
        };
        matches.push((rank, name.chars().count(), id));
    };
    if let Some(bible) = bible {
        for book in bible.books() {
            consider(book.book_id, &book.name);
        }
    }
    for (i, &name) in ENGLISH_BOOK_NAMES.iter().enumerate() {
        let id = i as i32 + 1;
        if !available(id) {
            continue;
        }
        consider(id, name);
        consider(id, CHINESE_BOOK_NAMES[i]);
        if ENGLISH_ALIASES[i]
            .split('|')
            .any(|alias| compact_name(alias) == query)
        {
            consider(id, token);
        }
    }
    matches.sort_unstable();
    if matches.first().is_some_and(|m| m.0 == 0) {
        return Some(matches[0].2);
    }
    let mut initials = initials.into_iter().filter(|&id| available(id));
    if let (Some(id), None) = (initials.next(), initials.next()) {
        return Some(id);
    }
    let &(rank, _, id) = matches.first()?;
    if require_unique_name && matches.iter().any(|m| m.0 == rank && m.2 != id) {
        return None;
    }
    Some(id)
}

fn positive(value: &str) -> Option<i32> {
    value.parse::<i32>().ok().filter(|&n| n > 0)
}

fn from_numbers(book_id: i32, parts: &Captures<'_>, quick: bool) -> Option<SmartReference> {
    let chapter = Some(positive(&parts[2])?);
    let mut verse_start = None;
    let mut verse_end = None;
    if let Some(first) = parts.get(3) {
        let first = positive(first.as_str())?;
        if let Some(last) = parts.get(4) {
            let last = positive(last.as_str())?;
            verse_start = Some(first.min(last));
            verse_end = Some(first.max(last));
        } else {
            verse_start = Some(first);
            verse_end = quick.then_some(first);
        }
    }
    Some(SmartReference {
        book_id,
        chapter,
        verse_start,
        verse_end,
    })
}

fn parse(input: &str, bible: Option<&Bible>, current_book: Option<i32>) -> Option<SmartReference> {
    let text = normalized(input);
    if let Some(parts) = REFERENCE.captures(text.trim()) {
        let token = parts[1].trim();
        let id = if token.is_empty() {
            current_book?
        } else {
            book_for(token, bible, false)?
        };
        return from_numbers(id, &parts, false);
    }
    let book_id = book_for(&text, bible, true)?;
    Some(SmartReference {
        book_id,
        chapter: None,
        verse_start: None,
        verse_end: None,
    })
}

pub fn parse_reference(input: &str) -> Option<SmartReference> {
    parse(input, None, None)
}

fn passages(input: &str, bible: Option<&Bible>) -> Option<Vec<SmartReference>> {
    let text = normalized(input);
    let mut remaining = text.as_str().trim();
    let mut result = Vec::new();
    while !remaining.is_empty() {
        let parts = PASSAGE.captures(remaining)?;
        let id = book_for(&parts[1], bible, false)?;
        result.push(from_numbers(id, &parts, true)?);
        remaining = remaining[parts.get(0)?.end()..].trim_start();
    }
    (!result.is_empty()).then_some(result)
}

pub fn quick_passage_references(input: &str) -> Option<Vec<SmartReference>> {
    passages(input, None)
}

fn passage_label(input: &str, bible: Option<&Bible>) -> Option<String> {
    let mut labels = Vec::new();
    for r in passages(input, bible)? {
        let book: &str = match bible {
            Some(b) => &b.book(r.book_id)?.name,
            None => CHINESE_BOOK_NAMES.get((r.book_id - 1) as usize)?,
        };
        let (chapter, first, last) = (r.chapter?, r.verse_start?, r.verse_end?);
        let verses = if first == last {
            first.to_string()
        } else {
            format!("{first}-{last}")
        };
        labels.push(format!("{book} {chapter}章{verses}节"));
    }
    Some(labels.join("；"))
}

pub fn quick_passage_label(input: &str) -> Option<String> {
    passage_label(input, None)
}

impl Bible {
    /// Resolve names in this module and interpret bookless references in the current book.
    pub fn parse_reference(&self, input: &str, current_book: i32) -> Option<SmartReference> {
        parse(
            input,
            Some(self),
            self.book(current_book).map(|book| book.book_id),
        )
    }

    pub fn quick_passage_references(&self, input: &str) -> Option<Vec<SmartReference>> {
        passages(input, Some(self))
    }

    pub fn quick_passage_label(&self, input: &str) -> Option<String> {
        passage_label(input, Some(self))
    }
}
