use crate::SelectedVerse;
use std::collections::HashMap;

fn verse_span(verse: &SelectedVerse) -> (i32, i32) {
    let mut numbers = verse
        .verse_range
        .split([',', '-'])
        .filter_map(|n| n.trim().parse::<i32>().ok());
    let first = numbers.next().unwrap_or(verse.verse_number);
    numbers.fold((first, first), |(lo, hi), n| (lo.min(n), hi.max(n)))
}

/// Accumulate a passage (one entry per translation) in presentation order.
pub fn append_passage(existing: &mut Vec<Vec<SelectedVerse>>, verses: Vec<SelectedVerse>) {
    if verses.is_empty() || existing.last() == Some(&verses) {
        return;
    }
    for previous in existing.iter_mut() {
        previous.retain(|single| {
            let (first, last) = verse_span(single);
            first != last
                || !verses.iter().any(|new| {
                    let (start, end) = verse_span(new);
                    start < end
                        && (start..=end).contains(&first)
                        && single.translation_file_name == new.translation_file_name
                        && single.bible_name == new.bible_name
                        && single.book_name == new.book_name
                        && single.chapter == new.chapter
                })
        });
    }
    existing.retain(|group| !group.is_empty());
    existing.push(verses);
}

/// Combine each translation's passages into one projection entry. The last passage supplies
/// its metadata, while continuous_passages retains the original grouping for navigation.
pub fn publish_accumulated_passages(passages: &[Vec<SelectedVerse>]) -> Vec<SelectedVerse> {
    let mut positions = HashMap::new();
    let mut groups: Vec<Vec<&SelectedVerse>> = Vec::new();
    for verse in passages.iter().flatten() {
        let key = if verse.translation_file_name.trim().is_empty() {
            &verse.bible_name
        } else {
            &verse.translation_file_name
        };
        let position = *positions.entry(key).or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        groups[position].push(verse);
    }
    groups
        .into_iter()
        .map(|group| {
            let last = *group.last().unwrap();
            let verse_text = group
                .iter()
                .map(|v| {
                    let range = if v.verse_range.trim().is_empty() {
                        v.verse_number.to_string()
                    } else {
                        v.verse_range.clone()
                    };
                    format!("{} {}:{}\n{}", v.book_name, v.chapter, range, v.verse_text)
                })
                .collect::<Vec<_>>()
                .join("\n\n");
            SelectedVerse {
                continuous_passages: passages.to_vec(),
                verse_text,
                ..last.clone()
            }
        })
        .collect()
}
