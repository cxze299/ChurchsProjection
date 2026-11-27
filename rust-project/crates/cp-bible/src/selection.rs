use crate::{parse_verse_code, Bible, BibleVerse, SelectedVerse, SmartReference};
use std::collections::BTreeSet;

/// Mouse selection for one chapter. Rows are zero-based indexes into Bible::get_chapter.
#[derive(Debug, Clone, Default)]
pub struct VerseSelection {
    row_count: usize,
    anchor: usize,
    multi: BTreeSet<usize>,
    awaiting_second_click: bool,
}

impl VerseSelection {
    pub fn new(row_count: usize) -> Self {
        Self {
            row_count,
            ..Self::default()
        }
    }

    pub fn selected_rows(&self) -> Vec<usize> {
        if self.multi.is_empty() {
            if self.row_count == 0 {
                Vec::new()
            } else {
                vec![self.anchor]
            }
        } else {
            self.multi.iter().copied().collect()
        }
    }

    pub fn click_verse(&mut self, row: usize) {
        if row >= self.row_count {
            return;
        }
        if self.awaiting_second_click && row != self.anchor {
            self.shift_click_verse(row);
        } else {
            self.multi.clear();
            self.anchor = row;
            self.awaiting_second_click = true;
        }
    }

    pub fn ctrl_click_verse(&mut self, row: usize) {
        if row >= self.row_count {
            return;
        }
        self.awaiting_second_click = false;
        if self.multi.remove(&row) {
            if self.multi.is_empty() {
                self.anchor = row;
            }
        } else {
            if self.multi.is_empty() {
                self.multi.insert(self.anchor);
            }
            self.multi.insert(row);
            self.anchor = row;
        }
    }

    pub fn shift_click_verse(&mut self, row: usize) {
        if row >= self.row_count {
            return;
        }
        self.awaiting_second_click = false;
        self.multi = (self.anchor.min(row)..=self.anchor.max(row)).collect();
    }
}

fn range_text(first: i32, last: i32) -> String {
    if first == last {
        first.to_string()
    } else {
        format!("{first}-{last}")
    }
}

pub fn format_verse_range(numbers: &[i32]) -> String {
    let mut numbers = numbers.to_vec();
    numbers.sort_unstable();
    numbers.dedup();
    let mut numbers = numbers.into_iter();
    let Some(mut first) = numbers.next() else {
        return String::new();
    };
    let mut last = first;
    let mut ranges = Vec::new();
    for number in numbers {
        if last.checked_add(1) != Some(number) {
            ranges.push(range_text(first, last));
            first = number;
        }
        last = number;
    }
    ranges.push(range_text(first, last));
    ranges.join(", ")
}

/// Return the primary passage followed by each parallel translation that contains a match.
/// Consecutive rows form a single range even in a translation with gaps in its numbering.
pub fn resolve_selection(
    primary: &Bible,
    parallel: &[&Bible],
    book_id: i32,
    chapter: i32,
    rows: &[usize],
) -> Vec<SelectedVerse> {
    let verses = primary.get_chapter(book_id, chapter);
    resolve_rows(primary, parallel, &verses, rows)
}

/// Resolve every quick passage before the caller appends any of them. A missing verse in
/// the primary translation invalidates the request, including gaps inside a typed range.
pub fn resolve_quick_passages(
    primary: &Bible,
    parallel: &[&Bible],
    references: &[SmartReference],
) -> Option<Vec<Vec<SelectedVerse>>> {
    references
        .iter()
        .map(|reference| {
            let chapter = primary.get_chapter(reference.book_id, reference.chapter?);
            let start = reference.verse_start?;
            let end = reference.verse_end.unwrap_or(start);
            if start <= 0 || end < start {
                return None;
            }
            let mut numbers = BTreeSet::new();
            let mut rows = Vec::new();
            for (i, verse) in chapter.iter().enumerate() {
                if (start..=end).contains(&verse.verse_number) && numbers.insert(verse.verse_number)
                {
                    rows.push(i);
                }
            }
            if numbers.len() != (end - start + 1) as usize {
                return None;
            }
            let passage = resolve_rows(primary, parallel, &chapter, &rows);
            (!passage.is_empty()).then_some(passage)
        })
        .collect()
}

fn resolve_rows(
    primary: &Bible,
    parallel: &[&Bible],
    verses: &[BibleVerse],
    rows: &[usize],
) -> Vec<SelectedVerse> {
    let mut indices: Vec<_> = rows.iter().copied().filter(|&i| i < verses.len()).collect();
    indices.sort_unstable();
    indices.dedup();
    let selected: Vec<_> = indices.iter().map(|&i| &verses[i]).collect();
    let Some(&first) = selected.first() else {
        return Vec::new();
    };
    let multiple = selected.len() > 1;
    let range = if !multiple {
        String::new()
    } else {
        let numbers: Vec<_> = selected.iter().map(|v| v.verse_number).collect();
        if indices.windows(2).all(|w| w[1] - w[0] == 1) {
            range_text(
                *numbers.iter().min().unwrap(),
                *numbers.iter().max().unwrap(),
            )
        } else {
            format_verse_range(&numbers)
        }
    };

    let mut result = Vec::new();
    let primary_texts: Vec<_> = selected
        .iter()
        .map(|v| v.verse_text.as_str())
        .filter(|s| !s.is_empty())
        .collect();
    if !primary_texts.is_empty() {
        result.push(projection(
            primary,
            first,
            first.book_id,
            first.chapter,
            first.verse_number,
            &range,
            primary_texts.join(" "),
        ));
    }
    for &bible in parallel {
        // A merged display row is identified by its first raw verse, as in getCodeReference.
        let mut matches = selected.iter().filter_map(|v| {
            let code = v.verse_id.split(',').next()?;
            Some((code, bible.get_verse_details_by_code(code)?))
        });
        let Some((code, native)) = matches.next() else {
            continue;
        };
        let text = std::iter::once(native.verse_text.as_str())
            .chain(matches.map(|(_, v)| v.verse_text.as_str()))
            .collect::<Vec<_>>()
            .join(" ");
        let canonical_book = parse_verse_code(code).unwrap().0;
        // Parallel ranges share the primary citation. Single verses use the native numbering.
        let (ch, number) = if multiple {
            (first.chapter, first.verse_number)
        } else {
            (native.chapter, native.verse_number)
        };
        result.push(projection(
            bible,
            native,
            canonical_book,
            ch,
            number,
            &range,
            text,
        ));
    }
    result
}

fn projection(
    bible: &Bible,
    native: &BibleVerse,
    book_id: i32,
    chapter: i32,
    verse_number: i32,
    range: &str,
    verse_text: String,
) -> SelectedVerse {
    SelectedVerse {
        translation_file_name: bible.filename().into(),
        bible_abbreviation: bible.abbreviation().into(),
        bible_name: bible.title().into(),
        book_name: bible
            .book(native.book_id)
            .expect("each verse has a book")
            .name
            .clone(),
        book_id,
        chapter,
        verse_number,
        verse_text,
        verse_range: range.into(),
        ..SelectedVerse::default()
    }
}
