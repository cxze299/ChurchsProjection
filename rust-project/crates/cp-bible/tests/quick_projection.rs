use cp_bible::{
    append_passage, publish_accumulated_passages, resolve_quick_passages, Bible, SmartReference,
};

const PRIMARY_SPB: &str = concat!(
    "##Title: Primary English\n",
    "19 Psalms 150\n",
    "-----\n",
    "B019C023V001 19 22 1 primary one\n",
    "B019C023V002 19 22 2 primary two\n",
    "B019C023V004 19 22 4 primary four\n",
);

const CHINESE_SPB: &str = concat!(
    "##Title: Chinese Revision\n",
    "19 诗篇和合本 150\n",
    "-----\n",
    "B019C023V001 19 24 7 parallel one\n",
    "B019C023V002 19 24 8 parallel two\n",
    "B019C023V004 19 24 10 parallel four\n",
);

const MISSING_MIDDLE_SPB: &str = concat!(
    "19 Psalms 150\n",
    "-----\n",
    "B019C023V001 19 22 1 primary one\n",
    "B019C023V003 19 22 3 primary three\n",
);

#[test]
fn resolves_and_publishes_quick_passages_from_linked_modules() {
    let primary = Bible::parse_spb(PRIMARY_SPB, "primary.spb");
    let chinese = Bible::parse_spb(CHINESE_SPB, "chinese.spb");

    let references = primary
        .quick_passage_references("Psalms 22 1-2 Psalms 22 4")
        .expect("both quick passages should parse");
    let passages = resolve_quick_passages(&primary, &[&chinese], &references)
        .expect("all referenced primary verses should resolve");

    let mut accumulated = Vec::new();
    for passage in passages {
        append_passage(&mut accumulated, passage);
    }
    let published = publish_accumulated_passages(&accumulated);

    assert_eq!(published.len(), 2);
    assert_eq!(
        published[0].verse_text,
        "Psalms 22:1-2\nprimary one primary two\n\nPsalms 22:4\nprimary four"
    );
    assert_eq!(
        published[1].verse_text,
        "诗篇和合本 22:1-2\nparallel one parallel two\n\n诗篇和合本 24:10\nparallel four"
    );
    assert_eq!(published[0].continuous_passages, accumulated);
    assert_eq!(published[1].continuous_passages, accumulated);

    assert_eq!(
        chinese.quick_passage_label("诗篇和合本 24 7-8"),
        Some("诗篇和合本 24章7-8节".to_string())
    );
    assert_eq!(
        chinese.parse_reference("诗篇和合本 24:7", 19),
        Some(SmartReference {
            book_id: 19,
            chapter: Some(24),
            verse_start: Some(7),
            verse_end: None,
        })
    );
    assert_eq!(
        chinese.parse_reference("24:8", 19),
        Some(SmartReference {
            book_id: 19,
            chapter: Some(24),
            verse_start: Some(8),
            verse_end: None,
        })
    );
}

#[test]
fn rejects_incomplete_ranges_and_any_missing_passage_in_the_request() {
    let primary = Bible::parse_spb(PRIMARY_SPB, "primary.spb");
    let missing_middle_bible = Bible::parse_spb(MISSING_MIDDLE_SPB, "missing-middle.spb");

    let missing_middle = missing_middle_bible
        .quick_passage_references("Psalms 22 1-3")
        .expect("the range should parse");
    assert!(resolve_quick_passages(&missing_middle_bible, &[], &missing_middle).is_none());

    let missing_last = primary
        .quick_passage_references("Psalms 22 1 Psalms 22 5")
        .expect("both passages should parse");
    assert!(resolve_quick_passages(&primary, &[], &missing_last).is_none());

    let maximum_range = primary
        .quick_passage_references("Psalms 22 1-2147483647")
        .expect("the maximum i32 endpoint should parse");
    assert_eq!(maximum_range[0].verse_end, Some(i32::MAX));
    assert!(resolve_quick_passages(&primary, &[], &maximum_range).is_none());
}
