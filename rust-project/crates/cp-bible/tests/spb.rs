use cp_bible::{
    extract_bible_abbreviation, generate_abbreviation, Bible, SearchScope, CHINESE_BOOK_NAMES,
    ENGLISH_BOOK_NAMES,
};
use regex::Regex;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const SPB_FIXTURE: &str = concat!(
    "##Title: King James Version (KJV)\n",
    "##Language: en\n",
    "19 Psalms 150\n",
    "1 Genesis 50\n",
    "-----\n",
    "B019C023V001 19 22 1 My\n",
    "B019C023V002 19 22 1 shepherd\n",
    "B019C023V003 19 22 2 The Lord is my shepherd\n",
    "B019C023V004 19 22 3 Grace abounds here\n",
    "B019C023V005\n",
    "Legacy first line\n",
    "second line\n",
    "B019C023V006 19 22 4 Faithful mercy\n",
    "B019C023V009 19 22 8 Eighth verse for fallback\n",
    "B019C023V010 19 22 10 起初，神创造天地。\n",
    "B001C001V001 1 1 1 In the beginning\n",
    "B001C001V002 1 1 2 A.B is a literal token\n",
    "B001C001V003 1 1 3 Axb is not the same token\n",
    "B001C003V001 1 3 1 Creation restored\n",
    "B002C004V001 2 4 1 Exodus verse only\n",
    "B019C023V007\n",
    "Last legacy line"
);

fn parsed_fixture() -> Bible {
    Bible::parse_spb(SPB_FIXTURE, "king-james.spb")
}

#[test]
fn parses_header_order_utf8_and_observed_chapter_counts() {
    let bible = parsed_fixture();
    let books = bible.books();

    assert_eq!(
        books.iter().map(|book| book.book_id).collect::<Vec<_>>(),
        vec![19, 1, 2]
    );
    assert_eq!(
        books
            .iter()
            .map(|book| book.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Psalms", "Genesis", "Exodus"]
    );
    assert_eq!(
        books
            .iter()
            .map(|book| book.chapter_count)
            .collect::<Vec<_>>(),
        vec![23, 3, 4]
    );
    assert_eq!(bible.title(), "King James Version (KJV)");
    assert_eq!(bible.abbreviation(), "KJV");

    let utf8_verse = bible
        .get_verse_details_by_code("B019C023V010")
        .expect("modern verse with UTF-8 text should load");
    assert_eq!(utf8_verse.verse_text, "起初，神创造天地。");
}

#[test]
fn keeps_duplicate_verse_parts_raw_and_merges_them_for_chapter_display() {
    let bible = parsed_fixture();
    let raw = bible.get_chapter_verses(19, 22);

    assert_eq!(raw.len(), 7);
    assert_eq!(raw[0].verse_number, 1);
    assert_eq!(raw[1].verse_number, 1);
    assert_eq!(raw[0].verse_id, "B019C023V001");
    assert_eq!(raw[1].verse_id, "B019C023V002");
    assert_eq!(raw[0].verse_text, "My");
    assert_eq!(raw[1].verse_text, "shepherd");

    let display = bible.get_chapter(19, 22);
    assert_eq!(display.len(), 6);
    assert_eq!(display[0].verse_number, 1);
    assert_eq!(display[0].verse_id, "B019C023V001,B019C023V002");
    assert_eq!(display[0].verse_text, "My shepherd");
}

#[test]
fn parses_legacy_blocks_between_modern_rows_and_flushes_the_last_block_at_eof() {
    let bible = parsed_fixture();
    let legacy_chapter = bible.get_chapter_verses(19, 23);

    assert_eq!(legacy_chapter.len(), 2);
    assert_eq!(legacy_chapter[0].verse_number, 5);
    assert_eq!(
        legacy_chapter[0].verse_text,
        "Legacy first line\nsecond line"
    );
    assert_eq!(legacy_chapter[1].verse_number, 7);
    assert_eq!(legacy_chapter[1].verse_text, "Last legacy line");
}

#[test]
fn looks_up_exact_codes_and_falls_back_through_the_internal_chapter_mapping() {
    let bible = parsed_fixture();
    let exact = bible
        .get_verse_details_by_code("B019C023V002")
        .expect("the exact internal code should resolve");
    assert_eq!(exact.verse_id, "B019C023V002");
    assert_eq!(exact.chapter, 22);
    assert_eq!(exact.verse_number, 1);
    assert_eq!(exact.verse_text, "shepherd");

    // V008 is absent as an exact code, while the internal chapter maps to display chapter 22.
    // The display verse 8 is stored under V009 in this deliberately non-matching fixture.
    let fallback = bible
        .get_verse_details_by_code("B019C023V008")
        .expect("missing exact verse code should use the chapter mapping fallback");
    assert_eq!(fallback.chapter, 22);
    assert_eq!(fallback.verse_number, 8);
    assert_eq!(fallback.verse_text, "Eighth verse for fallback");
    assert!(bible
        .get_verse_details_by_code("not-a-verse-code")
        .is_none());
}

#[test]
fn supports_regex_and_literal_keyword_search_with_book_and_chapter_scopes() {
    let bible = parsed_fixture();

    let regex = Regex::new("(?i)grace").unwrap();
    let results = bible.search_bible(&regex, SearchScope::Chapter(19, 22));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].book, "Psalms");
    assert_eq!((results[0].chapter, results[0].verse), (22, 3));
    assert!(results[0].verse_text.starts_with("Psalms 22:3 "));

    let beginning = bible.search_bible(
        &Regex::new("beginning|mercy").unwrap(),
        SearchScope::Book(1),
    );
    assert_eq!(beginning.len(), 1);
    assert_eq!(beginning[0].verse_text, "Genesis 1:1 In the beginning");

    let all_words = bible.search_keywords("FAITHFUL mercy", true, SearchScope::Chapter(19, 22));
    assert_eq!(all_words.len(), 1);
    assert_eq!(
        (all_words[0].book.as_str(), all_words[0].verse),
        ("Psalms", 4)
    );

    let phrase = bible.search_keywords("IN THE BEGINNING", false, SearchScope::Book(1));
    assert_eq!(phrase.len(), 1);
    assert_eq!(phrase[0].verse, 1);

    let literal = bible.search_keywords("a.b", false, SearchScope::Book(1));
    assert_eq!(literal.len(), 1);
    assert_eq!(literal[0].verse, 2);
    assert_eq!(literal[0].verse_text, "Genesis 1:2 A.B is a literal token");
    assert!(bible
        .search_keywords(".*", false, SearchScope::Book(1))
        .is_empty());
}

#[test]
fn derives_abbreviations_and_uses_default_book_names() {
    assert_eq!(
        extract_bible_abbreviation(Some("King James Version (KJV)"), "fallback.spb"),
        "KJV"
    );
    assert_eq!(
        extract_bible_abbreviation(Some("KJV."), "fallback.spb"),
        "KJV"
    );
    assert_eq!(
        extract_bible_abbreviation(None, r"C:\Bibles\ru_RST77.spb"),
        "ru_RST77"
    );
    assert_eq!(generate_abbreviation("Genesis"), "Gen");
    assert_eq!(generate_abbreviation("1 Corinthians"), "1 Cor");
    assert_eq!(generate_abbreviation("Song of Solomon"), "Song");

    assert_eq!(ENGLISH_BOOK_NAMES[0], "Genesis");
    assert_eq!(CHINESE_BOOK_NAMES[0], "创世记");

    let bible = Bible::parse_spb(
        "1 Book 1 50\n-----\nB001C001V001 1 1 1 起初，神创造天地。",
        "default-names.spb",
    );
    assert_eq!(bible.books()[0].name, "Genesis");
    assert_eq!(bible.books()[0].abbreviation, "Gen");
    assert_eq!(bible.abbreviation(), "default-names");
}

#[test]
fn loads_utf8_spb_text_from_a_file() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("cp-bible-spb-{}-{nonce}.spb", std::process::id()));
    fs::write(&path, SPB_FIXTURE).expect("write temporary SPB fixture");

    let bible = Bible::load_from_spb(path.as_path()).expect("load SPB fixture from disk");
    let verse = bible
        .get_verse_details_by_code("B019C023V010")
        .expect("UTF-8 verse should be available after file loading");
    assert_eq!(verse.verse_text, "起初，神创造天地。");

    fs::remove_file(path).expect("remove temporary SPB fixture");
}
