//! Contiguous initials search, including alternate Chinese pronunciations.
use pinyin::ToPinyinMulti;

fn initial_bit(letter: u8) -> u32 {
    if letter.is_ascii_alphabetic() {
        1 << (letter.to_ascii_lowercase() - b'a')
    } else {
        0
    }
}

fn initials(ch: char) -> u32 {
    if ch.is_ascii() {
        return initial_bit(ch as u8);
    }
    ch.to_pinyin_multi()
        .into_iter()
        .flatten()
        .fold(0, |bits, reading| {
            bits | initial_bit(reading.plain().as_bytes()[0])
        })
}

/// Matches a contiguous sequence of Pinyin initials or literal ASCII letters.
pub fn matches(title: &str, query: &str) -> bool {
    let mut query_bits = Vec::new();
    for ch in query.chars().filter(|ch| !ch.is_whitespace()) {
        if !ch.is_ascii_alphabetic() {
            return false;
        }
        query_bits.push(initial_bit(ch as u8));
    }
    if query_bits.is_empty() {
        return false;
    }
    let title_bits: Vec<_> = title
        .chars()
        .map(initials)
        .filter(|&bits| bits != 0)
        .collect();
    title_bits.windows(query_bits.len()).any(|window| {
        window
            .iter()
            .zip(&query_bits)
            .all(|(readings, letter)| readings & letter != 0)
    })
}
