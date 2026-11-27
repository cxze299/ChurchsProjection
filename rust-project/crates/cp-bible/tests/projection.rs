use cp_bible::{
    append_passage, format_verse_range, publish_accumulated_passages, resolve_selection, Bible,
    SelectedVerse, VerseSelection,
};

#[test]
fn ordinary_clicks_select_an_inclusive_range_then_start_a_new_selection() {
    let mut selection = VerseSelection::new(6);

    selection.click_verse(1);
    assert_eq!(selection.selected_rows(), vec![1]);

    selection.click_verse(4);
    assert_eq!(selection.selected_rows(), vec![1, 2, 3, 4]);

    selection.click_verse(2);
    assert_eq!(selection.selected_rows(), vec![2]);

    selection.click_verse(0);
    assert_eq!(selection.selected_rows(), vec![0, 1, 2]);
}

#[test]
fn ctrl_click_toggles_rows_and_shift_uses_the_latest_ctrl_selection_as_anchor() {
    let mut selection = VerseSelection::new(6);

    selection.click_verse(1);
    selection.ctrl_click_verse(4);
    assert_eq!(selection.selected_rows(), vec![1, 4]);

    selection.shift_click_verse(2);
    assert_eq!(selection.selected_rows(), vec![2, 3, 4]);

    selection.ctrl_click_verse(3);
    assert_eq!(selection.selected_rows(), vec![2, 4]);
}

#[test]
fn out_of_bounds_clicks_leave_the_selection_unchanged_including_for_empty_lists() {
    let mut selection = VerseSelection::new(3);
    selection.click_verse(1);
    selection.ctrl_click_verse(usize::MAX);
    selection.shift_click_verse(usize::MAX);
    selection.click_verse(3);
    assert_eq!(selection.selected_rows(), vec![1]);

    let mut empty = VerseSelection::new(0);
    empty.click_verse(0);
    empty.ctrl_click_verse(0);
    empty.shift_click_verse(0);
    assert!(empty.selected_rows().is_empty());
}

fn parse_bible(filename: &str, title: &str, verses: &[(&str, i32, i32, i32, &str)]) -> Bible {
    let mut text = format!("##Title: {title}\n19 Psalms 2\n-----\n");
    for (code, book_id, chapter, verse_number, verse_text) in verses {
        text.push_str(&format!(
            "{code} {book_id} {chapter} {verse_number} {verse_text}\n"
        ));
    }
    Bible::parse_spb(&text, filename)
}

fn primary_bible() -> Bible {
    parse_bible(
        "primary.spb",
        "Primary Text",
        &[
            ("B019C023V001", 19, 22, 1, "primary one"),
            ("B019C023V002", 19, 22, 3, "primary three"),
            ("B019C023V003", 19, 22, 5, "primary five"),
            ("B019C023V004", 19, 22, 6, "primary six"),
        ],
    )
}

fn parallel_bible() -> Bible {
    parse_bible(
        "parallel.spb",
        "Parallel Text",
        &[
            ("B019C023V001", 19, 24, 7, "parallel one"),
            ("B019C023V002", 19, 24, 17, "parallel three"),
            ("B019C023V003", 19, 24, 18, "parallel five"),
            ("B019C023V004", 19, 24, 19, "parallel six"),
        ],
    )
}

fn selected_verse(
    translation: (&str, &str, &str),
    book_name: &str,
    chapter: i32,
    verse_number: i32,
    verse_range: &str,
    verse_text: &str,
) -> SelectedVerse {
    let (filename, abbreviation, bible_name) = translation;
    SelectedVerse {
        translation_file_name: filename.to_string(),
        bible_abbreviation: abbreviation.to_string(),
        bible_name: bible_name.to_string(),
        book_name: book_name.to_string(),
        chapter,
        verse_number,
        verse_text: verse_text.to_string(),
        verse_range: verse_range.to_string(),
        book_id: 19,
        continuous_passages: Vec::new(),
    }
}

#[test]
fn multi_selection_uses_primary_native_numbers_for_parallel_references_and_metadata() {
    let primary = primary_bible();
    let parallel = parallel_bible();

    let selected = resolve_selection(&primary, &[&parallel], 19, 22, &[0, 1]);

    assert_eq!(selected.len(), 2);
    assert_eq!(
        selected[0],
        selected_verse(
            ("primary.spb", "PT", "Primary Text"),
            "Psalms",
            22,
            1,
            "1-3",
            "primary one primary three",
        )
    );
    assert_eq!(
        selected[1],
        selected_verse(
            ("parallel.spb", "PT", "Parallel Text"),
            "Psalms",
            22,
            1,
            "1-3",
            "parallel one parallel three",
        )
    );
}

#[test]
fn single_selection_uses_the_parallel_bibles_native_chapter_and_verse_numbers() {
    let primary = primary_bible();
    let parallel = parallel_bible();

    let selected = resolve_selection(&primary, &[&parallel], 19, 22, &[1]);

    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0].chapter, 22);
    assert_eq!(selected[0].verse_number, 3);
    assert_eq!(selected[0].verse_text, "primary three");
    assert_eq!(selected[1].translation_file_name, "parallel.spb");
    assert_eq!(selected[1].bible_abbreviation, "PT");
    assert_eq!(selected[1].bible_name, "Parallel Text");
    assert_eq!(selected[1].book_name, "Psalms");
    assert_eq!(selected[1].chapter, 24);
    assert_eq!(selected[1].verse_number, 17);
    assert_eq!(selected[1].verse_text, "parallel three");
    assert!(selected[1].verse_range.is_empty());
    assert_eq!(selected[1].book_id, 19);
}

#[test]
fn missing_parallel_references_are_skipped_without_losing_matching_selected_text() {
    let primary = primary_bible();
    let partial_parallel = parse_bible(
        "partial.spb",
        "Partial Text",
        &[("B019C023V002", 19, 24, 17, "partial three")],
    );
    let missing_parallel = parse_bible(
        "missing.spb",
        "Missing Text",
        &[("B019C023V004", 19, 24, 19, "unselected verse")],
    );

    let partial = resolve_selection(&primary, &[&partial_parallel], 19, 22, &[0, 1]);
    assert_eq!(partial.len(), 2);
    assert_eq!(partial[1].translation_file_name, "partial.spb");
    assert_eq!(partial[1].verse_text, "partial three");
    assert_eq!(partial[1].verse_range, "1-3");

    let missing = resolve_selection(&primary, &[&missing_parallel], 19, 22, &[0, 1]);
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].translation_file_name, "primary.spb");
}

#[test]
fn skipped_verse_numbers_and_disjoint_rows_keep_compact_ranges() {
    assert_eq!(format_verse_range(&[]), "");
    assert_eq!(format_verse_range(&[6, 2, 3, 3, 9, 8]), "2-3, 6, 8-9");

    let primary = primary_bible();
    let selected = resolve_selection(&primary, &[], 19, 22, &[0, 2, 3]);

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].verse_number, 1);
    assert_eq!(selected[0].verse_range, "1, 5-6");
    assert_eq!(
        selected[0].verse_text,
        "primary one primary five primary six"
    );
}

#[test]
fn append_passage_deduplicates_the_last_passage_and_preserves_insertion_order() {
    let first = selected_verse(("a.spb", "A", "Alpha"), "Psalms", 22, 1, "", "one");
    let second = selected_verse(("b.spb", "B", "Beta"), "Psalms", 22, 2, "", "two");
    let third = selected_verse(("c.spb", "C", "Gamma"), "Psalms", 22, 3, "", "three");
    let mut passages = vec![vec![first.clone()], vec![second.clone()]];

    append_passage(&mut passages, vec![second.clone()]);
    assert_eq!(passages, vec![vec![first.clone()], vec![second.clone()]]);

    append_passage(&mut passages, vec![third.clone()]);
    assert_eq!(passages, vec![vec![first], vec![second], vec![third]],);
}

#[test]
fn append_passage_removes_covered_single_verses_but_keeps_unrelated_passages_in_order() {
    let en_one = selected_verse(("en.spb", "EN", "English"), "Psalms", 22, 1, "", "one");
    let es_one = selected_verse(("es.spb", "ES", "Spanish"), "Psalms", 22, 1, "", "uno");
    let en_four = selected_verse(("en.spb", "EN", "English"), "Psalms", 22, 4, "", "four");
    let es_four = selected_verse(("es.spb", "ES", "Spanish"), "Psalms", 22, 4, "", "cuatro");
    let elsewhere = selected_verse(("en.spb", "EN", "English"), "John", 1, 1, "", "elsewhere");
    let en_range = selected_verse(("en.spb", "EN", "English"), "Psalms", 22, 1, "1-3", "range");
    let es_range = selected_verse(("es.spb", "ES", "Spanish"), "Psalms", 22, 1, "1-3", "rango");
    let mut passages = vec![
        vec![en_one, es_one],
        vec![en_four, es_four],
        vec![elsewhere.clone()],
    ];

    append_passage(&mut passages, vec![en_range.clone(), es_range.clone()]);

    assert_eq!(
        passages,
        vec![
            vec![
                selected_verse(("en.spb", "EN", "English"), "Psalms", 22, 4, "", "four"),
                selected_verse(("es.spb", "ES", "Spanish"), "Psalms", 22, 4, "", "cuatro"),
            ],
            vec![elsewhere],
            vec![en_range, es_range],
        ]
    );
}

#[test]
fn accumulated_passages_group_in_first_seen_order_and_keep_latest_translation_metadata() {
    let english_first = selected_verse(
        ("en.spb", "EN", "English"),
        "Genesis",
        1,
        1,
        "",
        "first passage",
    );
    let spanish_first = selected_verse(
        ("es.spb", "ES", "Spanish"),
        "Génesis",
        1,
        1,
        "",
        "primer pasaje",
    );
    let english_last = selected_verse(
        ("en.spb", "EN2", "English Revised"),
        "Psalms",
        2,
        3,
        "3-4",
        "second passage",
    );
    let spanish_last = selected_verse(
        ("es.spb", "ES2", "Spanish Revised"),
        "Salmos",
        2,
        3,
        "3-4",
        "segundo pasaje",
    );
    let passages = vec![
        vec![english_first, spanish_first],
        vec![english_last, spanish_last],
    ];

    let accumulated = publish_accumulated_passages(&passages);

    assert_eq!(accumulated.len(), 2);
    assert_eq!(accumulated[0].translation_file_name, "en.spb");
    assert_eq!(accumulated[0].bible_abbreviation, "EN2");
    assert_eq!(accumulated[0].bible_name, "English Revised");
    assert_eq!(accumulated[0].book_name, "Psalms");
    assert_eq!(accumulated[0].chapter, 2);
    assert_eq!(accumulated[0].verse_number, 3);
    assert_eq!(accumulated[0].verse_range, "3-4");
    assert_eq!(
        accumulated[0].verse_text,
        "Genesis 1:1\nfirst passage\n\nPsalms 2:3-4\nsecond passage"
    );
    assert_eq!(accumulated[0].continuous_passages, passages);

    assert_eq!(accumulated[1].translation_file_name, "es.spb");
    assert_eq!(accumulated[1].bible_abbreviation, "ES2");
    assert_eq!(accumulated[1].bible_name, "Spanish Revised");
    assert_eq!(
        accumulated[1].verse_text,
        "Génesis 1:1\nprimer pasaje\n\nSalmos 2:3-4\nsegundo pasaje"
    );
}
