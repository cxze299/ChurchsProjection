use cp_library::{discovery, pinyin_match, quick_play};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn write_file(path: &Path) -> PathBuf {
    fs::create_dir_all(path.parent().expect("test file has a parent")).unwrap();
    fs::write(path, b"test document").unwrap();
    path.to_path_buf()
}

#[test]
fn pinyin_search_accepts_case_and_query_whitespace() {
    assert!(pinyin_match::matches("001 奇异恩典", "QYED"));
    assert!(pinyin_match::matches("001 奇异恩典", "q y e d"));
    assert!(pinyin_match::matches("奇異恩典", "qyed"));
    assert!(pinyin_match::matches("1 至大医生现今可近", "zdy"));
    assert!(!pinyin_match::matches("奇异恩典", "zzzz"));
    assert!(!pinyin_match::matches("奇异恩典", "123"));
    assert!(!pinyin_match::matches("奇异恩典", "   "));
}

#[test]
fn pinyin_search_accepts_supported_polyphonic_readings() {
    let readings = [
        ("重", "z", "c"),
        ("乐", "l", "y"),
        ("朝", "z", "c"),
        ("行", "x", "h"),
        ("长", "z", "c"),
        ("奇", "q", "j"),
        ("调", "d", "t"),
    ];
    for (character, first_reading, second_reading) in readings {
        assert!(
            pinyin_match::matches(character, first_reading),
            "expected {character} to accept initial {first_reading}"
        );
        assert!(
            pinyin_match::matches(character, second_reading),
            "expected {character} to accept initial {second_reading}"
        );
    }
    assert!(pinyin_match::matches("重新得力", "cxdl"));
    assert!(pinyin_match::matches("重新得力", "zxdl"));
}

#[test]
fn discovery_recurses_across_roots_and_skips_temporary_unsupported_and_ignored_files() {
    let temp = tempdir().unwrap();
    let presentation_root = temp.path().join("presentations");
    let nested = presentation_root.join("BookA").join("Sunday");
    let additional_root = temp.path().join("additional");

    let expected = vec![
        write_file(&nested.join("message.PDF")),
        write_file(&nested.join("slides.pptx")),
        write_file(&additional_root.join("BookB").join("notes.docx")),
    ];

    write_file(&temp.path().join("outside.pdf"));
    write_file(&nested.join("~$notes.docx"));
    write_file(&nested.join("notes.txt"));
    write_file(&nested.join("node_modules").join("ignored.pdf"));
    write_file(&nested.join("build").join("ignored.pptx"));

    let roots = [
        presentation_root.as_path(),
        nested.as_path(),
        additional_root.as_path(),
        presentation_root.as_path(),
    ];
    let mut discovered = discovery::discover(&roots);
    let mut expected = expected;
    discovered.sort();
    expected.sort();

    assert_eq!(discovered, expected);
}

#[test]
fn discovery_returns_no_files_for_missing_roots() {
    let temp = tempdir().unwrap();
    let missing = temp.path().join("missing");

    assert!(discovery::discover(&[missing]).is_empty());
}

#[test]
fn quick_play_uses_the_first_directory_below_the_matching_root_as_book() {
    let temp = tempdir().unwrap();
    let primary = temp.path().join("hymns");
    let additional = temp.path().join("archive");
    let in_book = write_file(&primary.join("BookA").join("001 Hymn.pdf"));
    let at_root = write_file(&primary.join("002 Root Hymn.pdf"));
    let in_additional_book = write_file(&additional.join("BookB").join("003 Hymn.pdf"));
    let roots = [primary.as_path(), additional.as_path()];

    assert_eq!(quick_play::book(&in_book, &roots), "BookA");
    assert_eq!(quick_play::book(&at_root, &roots), "");
    assert_eq!(quick_play::book(&in_additional_book, &roots), "BookB");
}

#[test]
fn quick_play_ranks_preferred_formats_and_legacy_fallbacks() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("hymns");
    let roots = [root.as_path()];

    let amazing_pptx = write_file(&root.join("BookA").join("001 Amazing Grace.pptx"));
    let amazing_pdf = write_file(&root.join("BookA").join("001 Amazing Grace.pdf"));
    let book_b_amazing = write_file(&root.join("BookB").join("001 Amazing Grace.pptx"));
    let ten_hymn = write_file(&root.join("BookA").join("010 Ten Hymn.pptx"));
    let hundred_one_hymn = write_file(&root.join("BookA").join("101 Hundred One Hymn.pptx"));

    let legacy_ppt = write_file(&root.join("BookA").join("020 Legacy Hymn.ppt"));
    let legacy_pdf = write_file(&root.join("BookA").join("020 Legacy Hymn.pdf"));

    let legacy_doc = write_file(&root.join("BookA").join("030 Word Hymn.doc"));
    let word_pptx = write_file(&root.join("BookA").join("030 Word Hymn.pptx"));
    let word_pdf = write_file(&root.join("BookA").join("030 Word Hymn.pdf"));

    let fallback_pptx = write_file(&root.join("BookA").join("040 Fallback Hymn.pptx"));
    let fallback_pdf = write_file(&root.join("BookA").join("040 Fallback Hymn.pdf"));
    let song_file = write_file(&root.join("BookA").join("050 Song Format.song"));
    let song_pptx = write_file(&root.join("BookA").join("050 Song Format.pptx"));
    let song_pdf = write_file(&root.join("BookA").join("050 Song Format.pdf"));
    let files = vec![
        amazing_pptx.clone(),
        amazing_pdf.clone(),
        book_b_amazing,
        ten_hymn,
        hundred_one_hymn,
        legacy_ppt.clone(),
        legacy_pdf,
        legacy_doc.clone(),
        word_pptx,
        word_pdf,
        fallback_pptx.clone(),
        fallback_pdf,
        song_file.clone(),
        song_pptx,
        song_pdf,
    ];

    assert_eq!(
        quick_play::find(&files, &roots, "1", "BookA", "pptx"),
        vec![amazing_pptx],
    );
    assert_eq!(
        quick_play::find(&files, &roots, "001", "BookA", "pdf"),
        vec![amazing_pdf],
    );
    assert_eq!(
        quick_play::find(&files, &roots, "20", "BookA", "pptx"),
        vec![legacy_ppt],
    );
    assert_eq!(
        quick_play::find(&files, &roots, "30", "BookA", "docx"),
        vec![legacy_doc],
    );
    assert_eq!(
        quick_play::find(&files, &roots, "40", "BookA", "docx"),
        vec![fallback_pptx],
    );
    assert_eq!(
        quick_play::find(&files, &roots, "50", "BookA", "song"),
        vec![song_file],
    );
}

#[test]
fn quick_play_matches_ascii_substrings_case_insensitively_and_raw_contiguous_substrings() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("hymns");
    let roots = [root.as_path()];
    let english = write_file(&root.join("BookA").join("001 Amazing Grace.pptx"));
    let chinese = write_file(&root.join("BookA").join("002 奇异恩典.pdf"));

    assert_eq!(
        quick_play::find(
            std::slice::from_ref(&english),
            &roots,
            "GRACE",
            "BookA",
            "pptx"
        ),
        vec![english],
    );
    assert_eq!(
        quick_play::find(
            std::slice::from_ref(&chinese),
            &roots,
            "恩典",
            "BookA",
            "pdf"
        ),
        vec![chinese],
    );
}

#[test]
fn quick_play_searches_titles_by_pinyin_and_applies_book_filter() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("hymns");
    let roots = [root.as_path()];
    let book_a = write_file(&root.join("BookA").join("001 奇异恩典.pdf"));
    let book_b = write_file(&root.join("BookB").join("002 奇异恩典.pdf"));

    assert_eq!(
        quick_play::find(&[book_a.clone(), book_b], &roots, "q y e d", "BookA", "pdf"),
        vec![book_a],
    );
}
